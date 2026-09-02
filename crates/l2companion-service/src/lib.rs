//! # l2-core
//!
//! Character state management, domain models, and telemetry telemetry engine.

pub mod event;
#[cfg(feature = "server")]
pub mod graphql;
pub mod model;
#[cfg(feature = "server")]
pub mod server;
pub mod state;

pub use event::CompanionEvent;
#[cfg(feature = "server")]
pub use graphql::{build_schema, AppSchema};
pub use model::*;
#[cfg(feature = "server")]
pub use server::{create_router, start_api_server};
pub use state::CharacterTracker;
