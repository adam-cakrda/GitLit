use thiserror::Error;
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Missing Authorization header")]
    MissingAuthHeader,
    #[error("Invalid Authorization header")]
    InvalidAuthHeader,
    #[error("Unauthorized")]
    InvalidCredentials,
    #[error("Internal Server Error: {0}")]
    Internal(String),
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum GitError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("git failed: {0}")]
    Git(String),

    #[error("Invalid output: {0}")]
    Parse(String),
}

