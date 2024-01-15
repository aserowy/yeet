use bytes::Bytes;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{Constraint, CrosstermBackend, Direction, Layout, Terminal};
use std::{
    convert::Infallible,
    io::{stderr, stdout, BufWriter, Write},
    str,
};
use teywi_server::{Client, Error};

use crate::{state::{AppState, Message}, current_directory};

pub async fn run(address: String) -> Result<(), Error> {
    stderr().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.clear()?;

    let mut client = Client::connect(address).await?;
    client.set("foo", bytes_from_str("bar").unwrap()).await?;

    let mut state = AppState::default();

    loop {
        terminal.draw(|frame| {
            current_directory::update(&mut state, Message::Startup);

            // TODO: refactor layout
            let main = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(frame.size());

            let files = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
                .split(main[0]);

            current_directory::view(&mut state, frame, files[0]);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    if let Some(value) = client.get("foo").await? {
        if let Ok(string) = str::from_utf8(&value) {
            stdout()
                .lock()
                .write_all(format!("\"{}\"", string).as_bytes())?;
        } else {
            stdout()
                .lock()
                .write_all(format!("{:?}", value).as_bytes())?;
        }
    } else {
        stdout().lock().write_all(b"nil")?;
    }

    let _ = stdout().flush();

    Ok(())
}

fn bytes_from_str(src: &str) -> Result<Bytes, Infallible> {
    Ok(Bytes::from(src.to_string()))
}
