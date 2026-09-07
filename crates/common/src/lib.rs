//! Shared event and session types used by the Collector.
//!
//! JSON field names here are the HTTP/Kafka contract with the Go server
//! (`server/pkg/types`). Keep serde names in sync with those Go structs.

mod events;
mod session;

pub use events::{CatalogEvent, CatalogObjectType, SessionEvent, SessionEventType, SessionState};
pub use session::SessionRecord;
