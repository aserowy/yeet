use std::{env, path::PathBuf};

use model::register;
use update::commandline;

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

    let initial_path = get_initial_path(&settings.startup_path);
    let mut emitter = Emitter::start(initial_path.clone());

    let mut model = Model::default();
    register::init(&mut model.register, &mut emitter).await?;
    if history::cache::load(&mut model.history).is_err() {
        // TODO: add notifications in tui and show history load failed
    }

    let mut result = Vec::new();
    while let Some(messages) = emitter.receiver.recv().await {
        let size = terminal.size().expect("Failed to get terminal size");
        let layout = AppLayout::default(size, commandline::height(&model, &messages));

        let actions: Vec<_> = messages
            .iter()
            .flat_map(|message| update::update(&settings, &mut model, &layout, message))
            .flatten()
            .collect();

        if action::execute_pre_view(&actions, &mut emitter, &mut terminal).await? {
            view::view(&mut terminal, &mut model, &layout)?;
        }

        if !action::execute_post_view(&actions, &mut emitter, &model).await? {
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
