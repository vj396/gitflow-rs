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

    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// Result type alias to simplify function signatures
pub type Result<T> = std::result::Result<T, GitFlowError>;
