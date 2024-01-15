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
    event::{self, AppEvent},
    layout::AppLayout,
    model::Model,
    update::{self, Message},
    view::{current_directory, parent_directory},
};

pub async fn run(_address: String) -> Result<(), Error> {
    stderr().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.clear()?;

    let mut state = Model::default();
    let mut event_stream = event::start();

    while let Some(event) = event_stream.recv().await {
        match event {
            AppEvent::Error => todo!(),
            AppEvent::Key => todo!(),
            AppEvent::Mouse(_) => todo!(),
            AppEvent::Render => {
                terminal.draw(|frame| render(&mut state, frame))?;
            }
            AppEvent::Resize(_, _) => todo!(),
            AppEvent::Startup => update::update(&mut state, &Message::Refresh),
            AppEvent::Tick => update::update(&mut state, &Message::Refresh),
            AppEvent::Quit => break,
        }
    }

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn render(state: &mut Model, frame: &mut Frame) {
    let layout = AppLayout::default(frame.size());

    current_directory::view(state, frame, layout.current_directory);
    parent_directory::view(state, frame, layout.parent_directory);
}
