//! Shared event and session types used by Collector and Server.

mod events;
mod session;

pub use events::{CatalogEvent, CatalogObjectType, SessionEvent, SessionEventType, SessionState};
pub use session::SessionRecord;
