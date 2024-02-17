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
            // NOTE: arguments
            Arg::new("path")
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf))
                .help("path to open in yate on startup"),
            // NOTE: options
            Arg::new("stdout-on-open")
                .long("stdout-on-open")
                .action(ArgAction::SetTrue)
                .default_value("false")
                .help("on open print selected paths to stdout instead and close the application"),
        ])
}

fn map_args_to_settings(args: &ArgMatches, settings: &mut Settings) {
    settings.stdout_on_open = args.get_flag("stdout-on-open");
    settings.startup_path = args.get_one("path").map(|path: &PathBuf| path.clone());
}
