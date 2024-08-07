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
    Load(PathBuf, Option<String>),
    ModeChanged,
    Open(PathBuf),
    Quit(Option<String>),
    Resize(u16, u16),
    Task(Task),
    UnwatchPath(PathBuf),
    WatchPath(PathBuf),
}

#[derive(PartialEq)]
pub enum ActionResult {
    Normal,
    SkipRender,
    Quit,
}

#[tracing::instrument(skip(model, emitter, terminal, actions))]
pub async fn exec_preview_actions(
    model: &Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: &[Action],
) -> Result<ActionResult, AppError> {
    execute(true, model, emitter, terminal, actions).await
}

#[tracing::instrument(skip(model, emitter, terminal, actions))]
pub async fn exec_postview_actions(
    model: &Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: &[Action],
) -> Result<ActionResult, AppError> {
    execute(false, model, emitter, terminal, actions).await
}

fn is_preview_action(action: &Action) -> bool {
    match action {
        Action::Load(_, _)
        | Action::Open(_)
        | Action::Resize(_, _)
        | Action::Task(_)
        | Action::UnwatchPath(_)
        | Action::WatchPath(_) => true,

        Action::EmitMessages(_) | Action::ModeChanged | Action::Quit(_) => false,
    }
}

async fn execute(
    is_preview: bool,
    model: &Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: &[Action],
) -> Result<ActionResult, AppError> {
    for action in actions {
        if is_preview != is_preview_action(action) {
            continue;
        }

        tracing::debug!("handling action: {:?}", action);

        match action {
            Action::EmitMessages(messages) => {
                emitter.run(Task::EmitMessages(messages.clone()));
            }
            Action::Load(path, selection) => {
                if path.is_dir() {
                    emitter.run(Task::EnumerateDirectory(path.clone(), selection.clone()));
                } else {
                    emitter.run(Task::LoadPreview(path.clone()));
                }
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
                emitter.run(Task::SaveQuickFix(model.qfix.clone()));
            }
            Action::Resize(x, y) => {
                terminal.resize(*x, *y)?;
            }
            Action::Task(task) => emitter.run(task.clone()),
            Action::UnwatchPath(path) => {
                if path == &PathBuf::default() {
                    continue;
                }

                emitter.abort(&Task::EnumerateDirectory(path.clone(), None));

                if let Err(error) = emitter.unwatch(path.as_path()) {
                    tracing::debug!("emitting unwatch path failed: {:?}", error);
                }
            }
            Action::WatchPath(path) => {
                if path == &PathBuf::default() {
                    continue;
                }

                if let Err(error) = emitter.watch(path.as_path()) {
                    tracing::error!("emitting watch path failed: {:?}", error);
                }
            }
        }
    }

    let result = if is_preview && contains_emit(actions) {
        ActionResult::SkipRender
    } else if !is_preview && contains_quit(actions) {
        ActionResult::Quit
    } else {
        ActionResult::Normal
    };

    Ok(result)
}

fn contains_emit(actions: &[Action]) -> bool {
    actions.iter().any(|a| matches!(a, Action::EmitMessages(_)))
}

fn contains_quit(actions: &[Action]) -> bool {
    actions.iter().any(|a| matches!(a, Action::Quit(_)))
}
