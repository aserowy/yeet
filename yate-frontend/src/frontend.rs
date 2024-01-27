use crossterm::{
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    Frame,
};
use std::io::{stderr, BufWriter};
use yate_keymap::{message::Message, MessageResolver};

use crate::{
    event::{self, AppResult},
    layout::AppLayout,
    model::Model,
    update::{self},
    view::{self},
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn run(_address: String) -> Result<(), Error> {
    stderr().execute(EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.clear()?;

    let mut model = Model::default();
    let mut resolver = MessageResolver::default();

    let (_, mut receiver) = event::listen();
    while let Some(event) = receiver.recv().await {
        let messages = event::convert(event, &mut resolver);

        let mut result = Vec::new();
        terminal.draw(|frame| result = render(&mut model, frame, &messages))?;

        if result.contains(&AppResult::Quit) {
            break;
        }

        for app_result in result {
            match app_result {
                AppResult::ModeChanged(mode) => {
                    resolver.mode = mode;
                }
                _ => {}
            }
        }
    }

    stderr().execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn render(model: &mut Model, frame: &mut Frame, messages: &[Message]) -> Vec<AppResult> {
    let layout = AppLayout::default(frame.size());

    let app_results = messages
        .iter()
        .flat_map(|message| update::update(model, &layout, message))
        .collect();

    view::view(model, frame, &layout);

    app_results
}
