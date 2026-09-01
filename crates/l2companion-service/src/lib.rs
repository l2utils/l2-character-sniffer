//! # l2-core
//!
//! Character state management, domain models, and telemetry telemetry engine.

pub mod event;
pub mod model;
pub mod state;

pub use event::CompanionEvent;
pub use model::{Character, InventoryItem, Location, Stats, Vitals};
pub use state::CharacterTracker;
