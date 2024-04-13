use thiserror::Error;
use yeet_keymap::message::Envelope;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Sending render action failed")]
    ActionSendFailed(#[from] tokio::sync::mpsc::error::SendError<Envelope>),
    #[error("Error aggregation")]
    Aggregate(Vec<AppError>),
    #[error("File operation failed")]
    FileOperationFailed(#[from] std::io::Error),
    #[error("Path target is invalid")]
    InvalidTargetPath,
    #[error("Loading navigation history failed")]
    LoadHistoryFailed,
    #[error("Loading marks failed")]
    LoadMarkFailed,
    #[error("Loading quickfix failed")]
    LoadQuickFixFailed,
    #[error("Terminal not initialized")]
    TerminalNotInitialized,
    #[error("Watch operation on path failed")]
    WatchOperationFailed(#[from] notify::Error),
}
