use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Failed to get a connection from the pool: {0}")]
    ConnectionError(String),

    #[error("Failed to run query: {0}")]
    QueryError(String),
}

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Failed to run migrations: {0}")]
    MigrationFailed(String),
}

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("Database error: {0}")]
    DbError(#[from] DbError),

    #[error("User not found")]
    UserNotFound,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Application Database Error: {0}")]
    DbError(#[from] DbError),

    #[error("Validation Error: {0}")]
    ValidationError(String),

    #[error("Other Application Error: {0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum GrpcError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<GrpcError> for Status {
    fn from(err: GrpcError) -> Self {
        match err {
            GrpcError::InvalidArgument(msg) => Status::invalid_argument(msg),
            GrpcError::NotFound(msg) => Status::not_found(msg),
            GrpcError::AlreadyExists(msg) => Status::already_exists(msg),
            GrpcError::Internal(msg) => Status::internal(msg),
            GrpcError::Unknown(msg) => Status::unknown(msg),
        }
    }
}

impl From<RepoError> for GrpcError {
    fn from(err: RepoError) -> Self {
        match err {
            RepoError::DbError(e) => GrpcError::Internal(format!("Database error: {:?}", e)),
            RepoError::UserNotFound => GrpcError::NotFound("User not found".to_string()),
            RepoError::Unknown(e) => GrpcError::Unknown(format!("Unknown error: {}", e)),
        }
    }
}