pub mod commands;
pub mod logging;
pub mod models;
pub mod services;
pub mod utils;

// Re-exports with explicit namespaces to avoid conflicts
pub use commands::config;
pub use commands::workspace;
pub use commands::sidecar;
pub use commands::session as session_commands;

pub use logging::*;
pub use models::config as config_types;
pub use models::session;
pub use models::message;
pub use services::*;
