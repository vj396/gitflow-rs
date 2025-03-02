use thiserror::Error;

/// Custom error types for GitFlow
#[derive(Error, Debug)]
pub enum GitFlowError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Result type alias to simplify function signatures
pub type Result<T> = std::result::Result<T, GitFlowError>;
