//! Map LDAP entries to `CatalogEvent` and parse group CNs from `memberOf`.

use anyhow::{Context, Result};
use chrono::Utc;
use common::{CatalogEvent, CatalogObjectType};
use ldap3::SearchEntry;
use uuid::Uuid;

use crate::config::{AdConfig, LdapFlavor};

pub fn groups_from_member_of(values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter_map(|dn| cn_from_dn(dn))
        .collect()
}

pub fn cn_from_dn(dn: &str) -> Option<String> {
    dn.split(',')
        .find_map(|part| {
            let part = part.trim();
            part.strip_prefix("CN=")
                .or_else(|| part.strip_prefix("cn="))
                .map(str::to_string)
        })
}

// Find the username, SID, UPN, and memberOf attributes in the entry and 
// Fill in the CatalogEvent struct
pub fn map_user_entry(
    entry: &SearchEntry,
    ad: &AdConfig,
    tenant_id: &str,
) -> Result<CatalogEvent> {
    let attrs = &entry.attrs;
    let username = first_attr(attrs, username_keys(ad.ldap_flavor))
        .with_context(|| format!("missing username on {}", entry.dn))?;
    let sid = first_attr(attrs, sid_keys(ad.ldap_flavor)).unwrap_or_else(|| entry.dn.clone());
    let upn = first_attr(attrs, &["userPrincipalName", "mail"]);
    let member_of = entry
        .attrs
        .get("memberOf")
        .or_else(|| entry.attrs.get("memberof"))
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let groups = groups_from_member_of(member_of);

    let attributes = serde_json::json!({
        "dn": entry.dn,
        "upn": upn,
        "ldap_flavor": flavor_str(ad.ldap_flavor),
    });

    Ok(CatalogEvent {
        event_id: Uuid::new_v4(),
        tenant_id: tenant_id.to_string(),
        object_type: CatalogObjectType::User,
        sid,
        domain: ad.domain.clone(),
        name: username,
        groups,
        attributes,
        observed_at: Utc::now(),
    })
}

pub fn user_search_filter(ad: &AdConfig, usn_cursor: Option<&str>) -> String {
    match ad.ldap_flavor {
        LdapFlavor::Ad => {
            let disabled = "(!(userAccountControl:1.2.840.113556.1.4.803:=2))";
            let person = "(&(objectClass=user)(objectCategory=person))";
            if ad.use_usn_changed {
                if let Some(usn) = usn_cursor {
                    return format!("(&{person}{disabled}(uSNChanged>={usn}))");
                }
            }
            format!("(&{person}{disabled})")
        }
        LdapFlavor::Openldap => {
            if let Some(ts) = usn_cursor {
                // OpenLDAP: incremental via generalizedTime cursor (modifyTimestamp).
                return format!(
                    "(&(objectClass=inetOrgPerson)(modifyTimestamp>={}))",
                    ldap_escape_filter(ts)
                );
            }
            "(objectClass=inetOrgPerson)".to_string()
        }
    }
}

pub fn user_search_attrs(flavor: LdapFlavor) -> Vec<&'static str> {
    match flavor {
        LdapFlavor::Ad => vec![
            "sAMAccountName",
            "userPrincipalName",
            "objectSid",
            "memberOf",
            "distinguishedName",
            "uSNChanged",
        ],
        LdapFlavor::Openldap => vec!["uid", "cn", "mail", "memberOf", "entryUUID", "modifyTimestamp"],
    }
}

// What is USN changed? Documentation/Collector/AD_Service_Account.md: "USN changed"
// "Incremental sync via uSNChanged (AD) or modifyTimestamp (OpenLDAP)."
// This function is used to get the maximum USN changed value from the entries.
// The USN changed value is a 64-bit integer that is incremented by the AD whenever a user's attributes are changed.
// The USN changed value is used to determine the last time the user was synced.
pub fn max_usn_from_entries(entries: &[SearchEntry], flavor: LdapFlavor) -> Option<String> {
    match flavor {
        LdapFlavor::Ad => entries
            .iter()
            .filter_map(|e| first_attr(&e.attrs, &["uSNChanged"]))
            .max_by(|a, b| numeric_cmp(a, b))
            .map(|s| s.to_string()),
        LdapFlavor::Openldap => entries
            .iter()
            .filter_map(|e| first_attr(&e.attrs, &["modifyTimestamp"]))
            .max()
            .map(|s| s.to_string()),
    }
}

fn username_keys(flavor: LdapFlavor) -> &'static [&'static str] {
    match flavor {
        LdapFlavor::Ad => &["sAMAccountName"],
        LdapFlavor::Openldap => &["uid", "cn"],
    }
}

fn sid_keys(flavor: LdapFlavor) -> &'static [&'static str] {
    match flavor {
        LdapFlavor::Ad => &["objectSid"],
        LdapFlavor::Openldap => &["entryUUID"],
    }
}

fn flavor_str(flavor: LdapFlavor) -> &'static str {
    match flavor {
        LdapFlavor::Ad => "ad",
        LdapFlavor::Openldap => "openldap",
    }
}

fn first_attr(attrs: &std::collections::HashMap<String, Vec<String>>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(vals) = attrs.get(*key).or_else(|| attrs.get(&key.to_lowercase())) {
            if let Some(v) = vals.first() {
                return Some(v.clone());
            }
        }
    }
    None
}

fn numeric_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    match (a.parse::<u64>(), b.parse::<u64>()) {
        (Ok(x), Ok(y)) => x.cmp(&y),
        _ => a.cmp(b),
    }
}

fn ldap_escape_filter(value: &str) -> String {
    value
        .replace('\\', "\\5c")
        .replace('*', "\\2a")
        .replace('(', "\\28")
        .replace(')', "\\29")
        .replace('\0', "\\00")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_group_cns() {
        let groups = groups_from_member_of(&[
            "CN=Engineering,OU=Groups,DC=corp,DC=local".into(),
            "CN=VPN-Users,OU=Groups,DC=corp,DC=local".into(),
        ]);
        assert_eq!(groups, vec!["Engineering", "VPN-Users"]);
    }

    #[test]
    fn ad_filter_includes_disabled_bit() {
        let ad = AdConfig {
            enabled: true,
            domain: "corp.local".into(),
            ldap_uris: vec![],
            base_dn: "DC=corp,DC=local".into(),
            bind_dn: "cn=svc".into(),
            bind_password_ref: "X".into(),
            sync_interval_secs: 3600,
            page_size: 1000,
            use_usn_changed: true,
            ldap_flavor: LdapFlavor::Ad,
            allow_insecure_ldap: false,
        };
        let f = user_search_filter(&ad, None);
        assert!(f.contains("userAccountControl"));
        let inc = user_search_filter(&ad, Some("12345"));
        assert!(inc.contains("uSNChanged>=12345"));
    }
}
