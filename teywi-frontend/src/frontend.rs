use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    Frame,
};
use std::io::{stderr, BufWriter};
use teywi_server::Error;

use crate::{
    event::{self, Message},
    layout::AppLayout,
    state::AppState,
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
    let mut event_stream = event::start();

    while let Some(event) = event_stream.recv().await {
        match event {
            Message::Error => todo!(),
            Message::Key => todo!(),
            Message::Mouse(_) => todo!(),
            Message::Render => {
                terminal.draw(|frame| render(&mut state, frame))?;
            }
            Message::Resize(_, _) => todo!(),
            Message::Startup => update(&mut state, &Message::Startup),
            Message::Tick => update(&mut state, &Message::Tick),
            Message::Quit => break,
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

fn update(state: &mut AppState, message: &Message) {
    current_directory::update(state, message);
    parent_directory::update(state, message);
}

fn render(state: &mut AppState, frame: &mut Frame) {
    let layout = AppLayout::default(frame.size());

    current_directory::view(state, frame, layout.current_directory);
    parent_directory::view(state, frame, layout.parent_directory);
}

// fn bytes_from_str(src: &str) -> Result<Bytes, Infallible> {
//     Ok(Bytes::from(src.to_string()))
// }
