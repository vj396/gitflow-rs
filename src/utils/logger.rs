//! Module for initializing the application logger.
//!
//! This module sets the log verbosity based on the user-provided command line options.
//!
//! # Details
//! This module has been updated with detailed documentation and examples for easier maintenance.

use log::LevelFilter;
use tracing_subscriber::EnvFilter;

/// Initialize logging with the specified verbosity level
///
/// # Arguments
/// * `verbosity` - The verbosity level (0 for Info, 1 for Debug, and >=2 for Trace).
///
/// # Returns
/// * None
///
/// # Examples
/// ```rust
/// // Initialize logger with Debug verbosity.
/// init_logger(1);
/// ```
pub fn init_logger(verbosity: u8) {
    let log_level = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive(log_level.to_string().parse().unwrap()),
        )
        .init();
}
