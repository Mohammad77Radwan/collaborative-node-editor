use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuadtreeError {
    #[error("Node capacity exceeded")]
    CapacityExceeded,

    #[error("Invalid bounds")]
    InvalidBounds,

    #[error("Node not found: {0}")]
    NodeNotFound(u32),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, QuadtreeError>;
