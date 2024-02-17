use std::path::PathBuf;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use thiserror::Error;
use yate_frontend::{
    settings::Settings,
    tui::{self},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Application error")]
    AppError,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut frontend_settings = Settings::default();
    let args = cli().get_matches();
    map_args_to_settings(&args, &mut frontend_settings);

    match tui::run(frontend_settings).await {
        Ok(()) => Ok(()),
        Err(_) => Err(Error::AppError),
    }
}

fn cli() -> Command {
    Command::new("yate")
        .about("yate - yet another tui explorer")
        .args([
            Arg::new("stdout-on-open")
                .action(ArgAction::SetTrue)
                .default_value("false")
                .long("stdout-on-open")
                .help("on open print selected paths to stdout instead and close the application"),
            Arg::new("path")
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf))
                .help("path to open in yate on startup"),
        ])
}

fn map_args_to_settings(args: &ArgMatches, settings: &mut Settings) {
    settings.stdout_selection = args.get_flag("stdout-on-open");
    settings.startup_path = args
        .get_one("path")
        .and_then(|path: &PathBuf| Some(path.clone()));
}
