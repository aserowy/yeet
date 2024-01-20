use teywi_frontend::frontend::{self, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let port = 12341;
    let address = format!("127.0.0.1:{}", port);

    // let address_server = address.clone();
    // let server_handle: tokio::task::JoinHandle<Result<(), Error>> = tokio::spawn(async {
    //     let listener = TcpListener::bind(address_server).await?;
    //
    //     server::run(listener, signal::ctrl_c()).await;
    //
    //     Ok(())
    // });

    let frontend_handle: tokio::task::JoinHandle<Result<(), Error>> = tokio::spawn(async {
        frontend::run(address).await?;

        Ok(())
    });

    // tokio::join!(frontend_handle, server_handle).0?
    tokio::join!(frontend_handle).0?
}
