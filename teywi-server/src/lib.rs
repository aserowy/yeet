
pub mod clients;
pub use clients::{BlockingClient, BufferedClient, Client};

pub mod cmd;
pub use cmd::Command;

mod connection;
pub use connection::Connection;

pub mod frame;
pub use frame::Frame;

mod db;
use db::Db;
use db::DbDropGuard;

mod parse;
use parse::{Parse, ParseError};

pub mod server;

mod shutdown;
use shutdown::Shutdown;

pub const DEFAULT_PORT: u16 = 6379;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
