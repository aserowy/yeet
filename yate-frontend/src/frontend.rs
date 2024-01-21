use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal},
    Frame,
};
use std::io::{stderr, BufWriter};
use yate_keymap::{message::Message, MessageResolver};

use crate::{
    event::{self},
    layout::AppLayout,
    model::Model,
    update::{self},
    view::{self},
};

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn run(_address: String) -> Result<(), Error> {
    stderr().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(BufWriter::new(stderr())))?;
    terminal.clear()?;

    let mut model = Model::default();
    let mut message_resolver = MessageResolver::default();
    let (_, mut receiver) = event::listen_crossterm();

    while let Some(event) = receiver.recv().await {
        let messages = event::process_appevent(event, &mut message_resolver);
        terminal.draw(|frame| render(&mut model, frame, &messages))?;

        if messages.contains(&Message::Quit) {
            break;
        }
    }

    stderr().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn render(model: &mut Model, frame: &mut Frame, messages: &Vec<Message>) {
    let layout = AppLayout::default(frame.size());
    for message in messages {
        update::update(model, &layout, message);
    }

    view::view(model, frame, &layout);
}
