#[derive(Debug, thiserror::Error)]
pub enum TimersetError {
    #[error(transparent)]
    TaskSchedulerError(#[from] crate::task_scheduler::TaskSchedulerError),
    #[error(transparent)]
    EnvVarError(#[from] std::env::VarError),
    #[error(transparent)]
    SetLoggerError(#[from] log::SetLoggerError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("WindowsError: {0}")]
    WindowsError(std::io::Error),
    #[error(transparent)]
    Other(#[from] eyre::Report),
}

impl TimersetError {
    pub fn windows_error() -> Self {
        Self::WindowsError(std::io::Error::last_os_error())
    }
}

pub type TimersetResult<T> = Result<T, TimersetError>;
