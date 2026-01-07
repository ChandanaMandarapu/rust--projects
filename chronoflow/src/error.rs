use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChronoError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    
    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),
    
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Consensus error: {0}")]
    ConsensusError(String),
}

pub type Result<T> = std::result::Result<T, ChronoError>;