//! # l2-core
//!
//! Character state management, domain models, and sniffer telemetry engine.

pub mod event;
pub mod model;
pub mod state;

pub use event::SnifferEvent;
pub use model::{Character, InventoryItem, Location, Stats, Vitals};
pub use state::CharacterTracker;
