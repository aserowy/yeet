use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{stderr, BufWriter};
use teywi_server::Error;

use crate::{
    layout::AppLayout,
    state::{AppState, Message},
    views::{current_directory, parent_directory},
};

pub async fn run(_address: String) -> Result<(), Error> {
    stderr().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.clear()?;

    // let mut client = Client::connect(address).await?;
    // client.set("foo", bytes_from_str("bar").unwrap()).await?;

    let mut state = AppState::default();

    loop {
        terminal.draw(|frame| {
            let layout = AppLayout::default(frame.size());

            // NOTE: update phase
            current_directory::update(&mut state, Message::Startup);
            parent_directory::update(&mut state, Message::Startup);

            // NOTE: rendering phase
            current_directory::view(&mut state, frame, layout.current_directory);
            parent_directory::view(&mut state, frame, layout.parent_directory);
        })?;

        // TODO: handle input async and introduce fps/tick rates
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

    // if let Some(value) = client.get("foo").await? {
    //     if let Ok(string) = str::from_utf8(&value) {
    //         stdout()
    //             .lock()
    //             .write_all(format!("\"{}\"", string).as_bytes())?;
    //     } else {
    //         stdout()
    //             .lock()
    //             .write_all(format!("{:?}", value).as_bytes())?;
    //     }
    // } else {
    //     stdout().lock().write_all(b"nil")?;

    // let _ = stdout().flush();

    Ok(())
}

// fn bytes_from_str(src: &str) -> Result<Bytes, Infallible> {
//     Ok(Bytes::from(src.to_string()))
// }
