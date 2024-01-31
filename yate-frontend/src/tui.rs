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
    event::{self, PostRenderAction},
    layout::AppLayout,
    model::{
        history::{self},
        Model,
    },
    task::TaskManager,
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
    history::cache::load(&mut model.history);

    let mut resolver = MessageResolver::default();
    let mut tasks = TaskManager::default();

    let (_, mut receiver) = event::listen();
    while let Some(event) = receiver.recv().await {
        let messages = event::convert_to_messages(event, &mut resolver);

        let mut post_render_actions = Vec::new();
        terminal.draw(|frame| post_render_actions = render(&mut model, frame, &messages))?;

        if post_render_actions.contains(&PostRenderAction::Quit) {
            history::cache::save(&model.history);

            break;
        }

        for post_render_action in post_render_actions {
            match post_render_action {
                PostRenderAction::ModeChanged(mode) => resolver.mode = mode,
                PostRenderAction::Task(task) => tasks.run(task),
                PostRenderAction::Quit => unreachable!(),
            }
        }
    }

    tasks.finishing().await;

    stderr().execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn render(model: &mut Model, frame: &mut Frame, messages: &[Message]) -> Vec<PostRenderAction> {
    let layout = AppLayout::default(frame.size());

    let post_render_actions = messages
        .iter()
        .flat_map(|message| update::update(model, &layout, message))
        .flatten()
        .collect();

    view::view(model, frame, &layout);

    post_render_actions
}
