use thiserror::Error;

use crate::event::RenderAction;

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
    RenderActionSendFailed(#[from] tokio::sync::mpsc::error::SendError<RenderAction>),
}
