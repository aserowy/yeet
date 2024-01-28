use yate_frontend::tui::{self, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let port = 12341;
    let address = format!("127.0.0.1:{}", port);

    let frontend_handle: tokio::task::JoinHandle<Result<(), Error>> = tokio::spawn(async {
        tui::run(address).await?;

        Ok(())
    });

    tokio::join!(frontend_handle).0?
}
