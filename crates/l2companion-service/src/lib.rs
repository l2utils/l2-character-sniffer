//! # l2-core
//!
//! Character state management, domain models, and telemetry telemetry engine.

pub mod event;
pub mod model;
pub mod server;
pub mod state;

pub use event::CompanionEvent;
pub use model::*;
pub use server::{create_router, start_api_server};
pub use state::CharacterTracker;
