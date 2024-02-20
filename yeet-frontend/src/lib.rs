use std::{
    env,
    io::{stdout, Write},
    path::PathBuf,
    time::Duration,
};

use tokio::time;

use crate::{
    error::AppError,
    event::{Emitter, PostRenderAction, PreRenderAction, RenderAction},
    layout::AppLayout,
    model::{
        history::{self},
        Model,
    },
    settings::Settings,
    task::Task,
    terminal::TerminalWrapper,
};

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
    let mut model = Model::default();
    if history::cache::load(&mut model.history).is_err() {
        // TODO: add notifications in tui and show history load failed
    }

    let initial_path = get_initial_path(&settings.startup_path);
    let mut emitter = Emitter::start(initial_path.clone());

    let mut result = Vec::new();
    'app_loop: while let Some(messages) = emitter.receiver.recv().await {
        let size = terminal.size().expect("Failed to get terminal size");
        let layout = AppLayout::default(size);

        let render_actions: Vec<_> = messages
            .iter()
            .flat_map(|message| update::update(&settings, &mut model, &layout, message))
            .flatten()
            .collect();

        // TODO: refactor pre render actions
        let pre_render_actions = render_actions.iter().filter_map(|actn| match actn {
            RenderAction::Pre(pre) => Some(pre),
            _ => None,
        });

        for pre_render_action in pre_render_actions {
            match pre_render_action {
                PreRenderAction::Resize(x, y) => {
                    terminal.resize(*x, *y)?;
                }
                PreRenderAction::SleepBeforeRender => {
                    time::sleep(Duration::from_millis(25)).await;
                }
            }
        }

        view::view(&mut terminal, &mut model, &layout)?;

        // TODO: refactor post render actions
        let post_render_actions = render_actions.iter().filter_map(|actn| match actn {
            RenderAction::Post(post) => Some(post),
            _ => None,
        });

        for post_render_action in post_render_actions {
            match post_render_action {
                PostRenderAction::ModeChanged(mode) => {
                    emitter.set_current_mode(mode.clone()).await;
                }
                PostRenderAction::Open(path) => {
                    let path = path.clone();

                    // TODO: check with mime if suspend/resume is necessary?
                    match emitter.suspend().await {
                        Ok(result) => {
                            if !result {
                                continue;
                            }
                        }
                        Err(_err) => {} // TODO: log error
                    }

                    terminal.suspend();

                    // FIX: remove flickering (alternate screen leave and cli started)
                    open::path(&path).await?;

                    emitter.resume();
                    terminal.resume()?;

                    view::view(&mut terminal, &mut model, &layout)?;
                }
                PostRenderAction::Quit(stdout_result) => {
                    if let Some(stdout_result) = stdout_result {
                        stdout().lock().write_all(stdout_result.as_bytes())?;
                    }
                    break 'app_loop;
                }
                PostRenderAction::Task(task) => emitter.run(task.clone()),
                PostRenderAction::UnwatchPath(p) => {
                    if p == &PathBuf::default() {
                        continue;
                    }

                    emitter.abort(&Task::EnumerateDirectory(p.clone()));

                    if let Err(_error) = emitter.unwatch(p.as_path()) {
                        // TODO: log error
                    }
                }
                PostRenderAction::WatchPath(p) => {
                    if p == &PathBuf::default() {
                        continue;
                    }

                    if p.is_dir() {
                        emitter.run(Task::EnumerateDirectory(p.clone()));
                    } else {
                        emitter.run(Task::LoadPreview(p.clone()));
                    }

                    if let Err(_error) = emitter.watch(p.as_path()) {
                        // TODO: log error
                    }
                }
            }
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