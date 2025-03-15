//! Module for GitFlow error types.
//!
//! This module defines the custom error types for GitFlow and a convenient Result type alias.
//!
//! # Details
//! This module ensures all error cases are clearly defined to simplify error handling across the application.

use std::io;
use thiserror::Error;

/// Custom error types for GitFlow
#[derive(Error, Debug)]
pub enum GitFlowError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Operation aborted: {0}")]
    Aborted(String),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type alias to simplify function signatures
pub type Result<T> = std::result::Result<T, GitFlowError>;
