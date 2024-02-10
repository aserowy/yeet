use thiserror::Error;
use yate_frontend::tui::{self};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Application error")]
    AppError,
    #[error("Join handle failed: Subprocess killed without shutting down")]
    JoinHandleFailed,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let port = 12341;
    let address = format!("127.0.0.1:{}", port);

    let _result = tui::run(address).await;

    Ok(())
}
