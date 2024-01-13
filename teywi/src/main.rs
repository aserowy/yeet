use bytes::Bytes;
use crossterm::{
    // Bind a TCP listener
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use std::{
    convert::Infallible,
    io::{stderr, stdout, BufWriter, Write},
    str,
};
use teywi_server::{server, Client, Error};
use tokio::{net::TcpListener, signal};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let port = 12341;
    let address = format!("127.0.0.1:{}", port);

    let address_server = address.clone();
    let server_handle: tokio::task::JoinHandle<Result<(), Error>> = tokio::spawn(async {
        let listener = TcpListener::bind(address_server).await?;

        server::run(listener, signal::ctrl_c()).await;

        Ok(())
    });

    let frontend_handle = tokio::spawn(async {
        stderr().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
        terminal.clear()?;

        let mut client = Client::connect(address).await?;
        client.set("foo", bytes_from_str("bar").unwrap()).await?;

        loop {
            terminal.draw(|frame| {
                let area = frame.size();
                frame.render_widget(
                    Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                        .white()
                        .on_blue(),
                    area,
                );
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
    });

    tokio::join!(frontend_handle, server_handle).0?
}

fn bytes_from_str(src: &str) -> Result<Bytes, Infallible> {
    Ok(Bytes::from(src.to_string()))
}
