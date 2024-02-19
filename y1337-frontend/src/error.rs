use thiserror::Error;
use y1337_keymap::message::Message;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Error aggregation")]
    Aggregate(Vec<AppError>),
    #[error("File operation failed")]
    FileOperationFailed(#[from] std::io::Error),
    #[error("Path target is invalid")]
    InvalidTargetPath,
    #[error("Loading navigation history failed")]
    LoadHistoryFailed,
    #[error("Sending render action failed")]
    RenderActionSendFailed(#[from] tokio::sync::mpsc::error::SendError<Message>),
    #[error("Terminal not initialized")]
    TerminalNotInitialized,
}
