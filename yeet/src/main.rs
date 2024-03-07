use std::path::PathBuf;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use thiserror::Error;
use tracing::Level;
use yeet_frontend::settings::Settings;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Initialization error")]
    Initialization,
}

#[tokio::main]
async fn main() {
    let cli = cli().get_matches();

    if let Ok(logpath) = get_logging_path() {
        let loglevel = get_log_level(&cli);
        let logfile = tracing_appender::rolling::daily(logpath, "log");
        tracing_subscriber::fmt()
            .pretty()
            .with_max_level(loglevel)
            .with_file(false)
            .with_writer(logfile)
            .init();
    }

    tracing::info!("starting application");

    match yeet_frontend::run(get_settings(&cli)).await {
        Ok(()) => {
            tracing::info!("closing application");
        }
        Err(err) => {
            tracing::error!("closing application with error: {:?}", err);
        }
    }
}

fn cli() -> Command {
    Command::new("yeet")
        .about("yeet - yet another... read the name on gh...")
        .args([
            // NOTE: arguments
            Arg::new("path")
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf))
                .help("path to open in yeet on startup"),
            // NOTE: options
            Arg::new("stdout-on-open")
                .long("stdout-on-open")
                .action(ArgAction::SetTrue)
                .default_value("false")
                .help("on open print selected paths to stdout instead and close the application"),
            Arg::new("verbosity")
                .short('v')
                .long("verbosity")
                .default_value("warn")
                .value_parser(["error", "warn", "info", "debug", "trace"])
                .help("set verbosity level for file logging"),
        ])
}

fn get_log_level(args: &ArgMatches) -> Level {
    match args
        .get_one::<String>("verbosity")
        .expect("default for verbosity set")
        .as_str()
    {
        "error" => Level::ERROR,
        "warn" => Level::WARN,
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        _ => Level::WARN,
    }
}

fn get_logging_path() -> Result<String, Error> {
    let cache_dir = match dirs::cache_dir() {
        Some(cache_dir) => match cache_dir.to_str() {
            Some(cache_dir_string) => cache_dir_string.to_string(),
            None => return Err(Error::Initialization),
        },
        None => return Err(Error::Initialization),
    };

    Ok(format!("{}{}", cache_dir, "/yeet/logs"))
}

fn get_settings(args: &ArgMatches) -> Settings {
    Settings {
        stdout_on_open: args.get_flag("stdout-on-open"),
        startup_path: args.get_one("path").cloned(),
    }
}
