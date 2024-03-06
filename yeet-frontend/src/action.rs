use std::{
    io::{stdout, Write},
    path::PathBuf,
};

use yeet_keymap::message::Message;

use crate::{
    error::AppError, event::Emitter, model::Model, open, task::Task, terminal::TerminalWrapper,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    EmitMessages(Vec<Message>),
    ModeChanged,
    Open(PathBuf),
    Quit(Option<String>),
    Resize(u16, u16),
    SkipRender,
    Task(Task),
    UnwatchPath(PathBuf),
    WatchPath(PathBuf),
}

#[derive(Debug, PartialEq)]
pub enum ActionResult {
    Normal,
    SkipRender,
    Quit,
}

pub async fn pre(
    model: &Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: &[Action],
) -> Result<ActionResult, AppError> {
    execute(true, model, emitter, terminal, actions).await
}

pub async fn post(
    model: &Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: &[Action],
) -> Result<ActionResult, AppError> {
    execute(false, model, emitter, terminal, actions).await
}

fn is_preview_action(action: &Action) -> bool {
    match action {
        Action::EmitMessages(_) => false,
        Action::ModeChanged => false,
        Action::Open(_) => true,
        Action::Quit(_) => false,
        Action::Resize(_, _) => true,
        Action::SkipRender => true,
        Action::Task(_) => true,
        Action::UnwatchPath(_) => true,
        Action::WatchPath(_) => true,
    }
}

async fn execute(
    is_preview: bool,
    model: &Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: &[Action],
) -> Result<ActionResult, AppError> {
    let mut result = ActionResult::Normal;

    for action in actions {
        if is_preview != is_preview_action(action) {
            continue;
        }

        match action {
            Action::EmitMessages(messages) => {
                emitter.run(Task::EmitMessages(messages.clone()));
            }
            Action::ModeChanged => {
                emitter.set_current_mode(model.mode.clone()).await;
            }
            Action::Open(path) => {
                // TODO: check with mime if suspend/resume is necessary?
                match emitter.suspend().await {
                    Ok(result) => {
                        if !result {
                            continue;
                        }
                    }
                    Err(error) => {
                        tracing::error!("emitter suspend failed: {:?}", error);
                    }
                }

                terminal.suspend();

                // FIX: remove flickering (alternate screen leave and cli started)
                open::path(path).await?;

                emitter.resume();
                terminal.resume()?;
            }
            Action::Quit(stdout_result) => {
                if let Some(stdout_result) = stdout_result {
                    stdout().lock().write_all(stdout_result.as_bytes())?;
                }
                emitter.run(Task::SaveHistory(model.history.clone()));
                emitter.run(Task::SaveMarks(model.marks.clone()));

                result = ActionResult::Quit;
            }
            Action::Resize(x, y) => {
                terminal.resize(*x, *y)?;
            }
            Action::SkipRender => result = ActionResult::SkipRender,
            Action::Task(task) => emitter.run(task.clone()),
            Action::UnwatchPath(path) => {
                if path == &PathBuf::default() {
                    continue;
                }

                emitter.abort(&Task::EnumerateDirectory(path.clone()));

                if let Err(error) = emitter.unwatch(path.as_path()) {
                    tracing::error!("emitting unwatch path failed: {:?}", error);
                }
            }
            Action::WatchPath(path) => {
                if path == &PathBuf::default() {
                    continue;
                }

                if path.is_dir() {
                    emitter.run(Task::EnumerateDirectory(path.clone()));
                } else {
                    emitter.run(Task::LoadPreview(path.clone()));
                }

                if let Err(error) = emitter.watch(path.as_path()) {
                    tracing::error!("emitting watch path failed: {:?}", error);
                }
            }
        }
    }

    Ok(result)
}
