use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    Frame,
};
use std::io::{stderr, BufWriter};
use yate_keymap::{action::Action, ActionResolver};

use crate::{
    event::{self},
    layout::AppLayout,
    model::Model,
    update::{self},
    view::{commandline, current_directory, parent_directory},
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn run(_address: String) -> Result<(), Error> {
    stderr().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.clear()?;

    let mut model = Model::default();
    let mut action_resolver = ActionResolver::default();
    let (_, mut receiver) = event::listen_crossterm();

    while let Some(event) = receiver.recv().await {
        let action = event::process_appevent(event, &mut action_resolver);
        terminal.draw(|frame| render(&mut model, frame, &action))?;

        if action == Action::Quit {
            break;
        }
    }

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn render(model: &mut Model, frame: &mut Frame, message: &Action) {
    let layout = AppLayout::default(frame.size());
    update::update(model, &layout, message);

    current_directory::view(model, frame, layout.current_directory);
    parent_directory::view(model, frame, layout.parent_directory);
    commandline::view(model, frame, layout.commandline);
}
