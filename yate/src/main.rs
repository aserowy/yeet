use thiserror::Error;
use tokio::task::{JoinError, JoinHandle};
use yate_frontend::{
    error::AppError,
    tui::{self},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Application error: {0}")]
    AppError(#[from] AppError),
    #[error("Join handle failed: Subprocess killed without shutting down")]
    JoinHandleFailed(#[from] JoinError),
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let port = 12341;
    let address = format!("127.0.0.1:{}", port);

    let frontend_handle: JoinHandle<Result<(), AppError>> =
        tokio::spawn(async { tui::run(address).await });

    match tokio::join!(frontend_handle).0 {
        Ok(app_result) => match app_result {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::AppError(error)),
        },
        Err(error) => Err(Error::JoinHandleFailed(error)),
    }
}
