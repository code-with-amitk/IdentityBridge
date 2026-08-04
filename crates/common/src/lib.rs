//! Shared event and session types used by Collector and Server.

mod events;
mod session;

pub use events::{CatalogEvent, SessionEvent, SessionState};
pub use session::SessionRecord;
