use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderlyError {
    #[error("Config error: {0}")]
    Config(serde_yaml::Error),

    #[error("Directory does not exist: {0}")]
    DirectoryDoesNotExist(String),

    #[error("Failed to read directory {0}: {1}")]
    FailedToReadDirectory(String, std::io::Error),

    #[error("Potential infinite loop detected for file: {0}")]
    InfiniteLoop(String),

    #[error("Invalid action type: {0}")]
    InvalidActionType(String),

    #[error("Invalid path: {0:?}")]
    InvalidFile(Option<String>),

    #[error("Invalid path: {0:?}")]
    InvalidPath(Option<String>),

    #[error("Invalid pattern: {0:?}")]
    InvalidPattern(Option<String>),

    #[error("IO error: {0}")]
    IoError(std::io::Error),
}

impl<T> From<OrderlyError> for Result<T, Box<dyn std::error::Error>> {
    fn from(err: OrderlyError) -> Self {
        Err(err.into())
    }
}

impl From<std::io::Error> for OrderlyError {
    fn from(err: std::io::Error) -> Self {
        OrderlyError::IoError(err)
    }
}
