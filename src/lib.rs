pub mod cli;
pub mod commands;

/// Common Result type for the CLI
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
