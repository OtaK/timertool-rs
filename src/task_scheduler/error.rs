#[derive(Debug, thiserror::Error)]
pub enum TaskSchedulerError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Other(#[from] eyre::Report),
}

impl From<std::io::ErrorKind> for TaskSchedulerError {
    fn from(ek: std::io::ErrorKind) -> Self {
        std::io::Error::from(ek).into()
    }
}

pub type TaskSchedulerResult<T> = Result<T, TaskSchedulerError>;
