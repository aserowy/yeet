use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    Frame,
};
use std::io::{stderr, BufWriter};
use teywi_keymap::{action::Action, ActionResolver};
use teywi_server::Error;

use crate::{
    event::{self, AppEvent},
    layout::AppLayout,
    model::Model,
    update::{self},
    view::{commandline, current_directory, parent_directory},
};

pub async fn run(_address: String) -> Result<(), Error> {
    stderr().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.clear()?;

    let mut model = Model::default();
    let mut action_resolver = ActionResolver::default();
    let (sender, mut receiver) = event::start();

    while let Some(event) = receiver.recv().await {
        match event {
            AppEvent::Error => todo!(),
            AppEvent::Key(key) => {
                if let Some(action) = action_resolver.add_and_resolve(key) {
                    terminal.draw(|frame| render(&mut model, frame, &action))?;

                    if action == Action::Quit {
                        let _ = sender.send(AppEvent::Quit);
                    }
                } else {
                    let action = &Action::KeySequenceChanged(action_resolver.get_key_string());
                    terminal.draw(|frame| render(&mut model, frame, action))?;
                }
            }
            AppEvent::Resize(_, _) => todo!(),
            AppEvent::Startup => {
                terminal.draw(|frame| render(&mut model, frame, &Action::Refresh))?;
            }
            AppEvent::Quit => break,
        }
    }

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn render(model: &mut Model, frame: &mut Frame, message: &Action) {
    update::update(model, message);

    let layout = AppLayout::default(frame.size());
    current_directory::view(model, frame, layout.current_directory);
    parent_directory::view(model, frame, layout.parent_directory);
    commandline::view(model, frame, layout.statusline);
}
