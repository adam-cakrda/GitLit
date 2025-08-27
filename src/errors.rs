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
