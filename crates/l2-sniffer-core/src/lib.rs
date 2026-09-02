//! # l2-core
//!
//! Character state management, domain models, and sniffer telemetry engine.

pub mod event;
pub mod model;
pub mod server;
pub mod state;

pub use event::SnifferEvent;
pub use model::*;
pub use server::{create_router, start_api_server};
pub use state::CharacterTracker;
