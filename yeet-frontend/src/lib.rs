use std::{env, path::PathBuf};

use action::{Action, ActionResult};
use layout::CommandLineLayout;
use model::{mark, qfix, register};
use task::Task;
use update::model::commandline;
use yeet_keymap::message::{Buffer, Message, Mode, PrintContent};

use crate::{
    error::AppError,
    event::Emitter,
    layout::AppLayout,
    model::{
        history::{self},
        Model,
    },
    settings::Settings,
    terminal::TerminalWrapper,
};

mod action;
pub mod error;
mod event;
mod layout;
mod model;
mod open;
pub mod settings;
mod task;
mod terminal;
mod update;
mod view;

pub async fn run(settings: Settings) -> Result<(), AppError> {
    let mut terminal = TerminalWrapper::start()?;
    let mut emitter = Emitter::start();

    let initial_path = get_initial_path(&settings.startup_path);
    emitter.run(Task::EmitMessages(vec![
        Message::Buffer(Buffer::ChangeMode(Mode::Normal, Mode::default())),
        Message::NavigateToPath(initial_path),
    ]));

    let mut model = Model {
        settings,
        ..Default::default()
    };

    register::file::init(&mut model.junk, &mut emitter).await?;

    if history::cache::load(&mut model.history).is_err() {
        emitter.run(Task::EmitMessages(vec![Message::Print(vec![
            PrintContent::Error("Failed to load history".to_string()),
        ])]));
    }

    if mark::load(&mut model.marks).is_err() {
        emitter.run(Task::EmitMessages(vec![Message::Print(vec![
            PrintContent::Error("Failed to load marks".to_string()),
        ])]));
    }

    if qfix::load(&mut model.qfix).is_err() {
        emitter.run(Task::EmitMessages(vec![Message::Print(vec![
            PrintContent::Error("Failed to load qfix".to_string()),
        ])]));
    }

    let mut result = Vec::new();
    while let Some(messages) = emitter.receiver.recv().await {
        tracing::debug!("received messages: {:?}", messages);

        let size = terminal.size().expect("Failed to get terminal size");
        model.layout = AppLayout::new(size, commandline::height(&model, &messages));

        let sequence_len = model.key_sequence.chars().count() as u16;
        model.commandline.layout = CommandLineLayout::new(model.layout.commandline, sequence_len);

        let mut actions: Vec<_> = messages
            .iter()
            .flat_map(|message| update::update(&mut model, message))
            .collect();

        let result = action::pre(&model, &mut emitter, &mut terminal, &actions).await?;
        if result != ActionResult::SkipRender {
            view::view(&mut terminal, &mut model)?;
        }

        actions.extend(get_watcher_changes(&mut model));

        let result = action::post(&model, &mut emitter, &mut terminal, &actions).await?;
        if result == ActionResult::Quit {
            break;
        }
    }

    if let Err(error) = emitter.shutdown().await {
        result.push(error);
    }

    terminal.shutdown()?;

    if result.is_empty() {
        Ok(())
    } else {
        Err(AppError::Aggregate(result))
    }
}

fn get_initial_path(initial_selection: &Option<PathBuf>) -> PathBuf {
    if let Some(path) = initial_selection {
        if path.exists() {
            return path.to_path_buf();
        }
    }

    env::current_dir().expect("Failed to get current directory")
}

#[tracing::instrument(skip(model))]
fn get_watcher_changes(model: &mut Model) -> Vec<Action> {
    let current = vec![
        Some(model.current.path.clone()),
        model.preview.path.clone(),
        model.parent.path.clone(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let mut actions = Vec::new();
    for path in &model.watches {
        if !current.contains(path) {
            actions.push(Action::UnwatchPath(path.clone()));
        }
    }

    for path in &current {
        if !model.watches.contains(path) {
            actions.push(Action::WatchPath(path.clone()));
        }
    }

    model.watches = current;

    if !actions.is_empty() {
        tracing::trace!("watcher changes: {:?}", actions);
    }

    actions
}
