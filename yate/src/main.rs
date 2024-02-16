use std::path::PathBuf;

use clap::Parser;
use thiserror::Error;
use yate_frontend::tui::{self};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Application error")]
    AppError,
}

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "yate")]
#[command(about = "yate", long_about = "yet another tui explorer")]
struct Cli {
    #[arg(required = false)]
    path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _args = Cli::parse();

    match tui::run(_args.path).await {
        Ok(()) => Ok(()),
        Err(_) => Err(Error::AppError),
    }
}
