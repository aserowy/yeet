use std::path::PathBuf;

use clap::Parser;
use thiserror::Error;
use yate_frontend::tui::{self};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Application error")]
    AppError,
}

#[derive(Debug, Parser)]
#[command(name = "yate")]
#[command(about = "yate - yet another tui explorer")]
struct Cli {
    #[arg(
        long = "stdout-on-open",
        help = "on open print selected paths to stdout instead and quit the application"
    )]
    stdout_on_open: bool,
    #[arg(help = "path to open in yate on startup")]
    path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();

    match tui::run(args.path).await {
        Ok(()) => Ok(()),
        Err(_) => Err(Error::AppError),
    }
}
