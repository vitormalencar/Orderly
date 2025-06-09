use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderlyError {
    #[error("Directory does not exist: {0}")]
    DirectoryDoesNotExist(String),

    #[error("Potential infinite loop detected for file: {0}")]
    InfiniteLoop(String),

    #[error("Config error: {0}")]
    Config(serde_yaml::Error),
}

impl<T> From<OrderlyError> for Result<T, Box<dyn std::error::Error>> {
    fn from(err: OrderlyError) -> Self {
        Err(err.into())
    }
}
