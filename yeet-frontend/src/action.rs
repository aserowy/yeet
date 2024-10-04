use std::{
    io::{stdout, Write},
    path::PathBuf,
};

use yeet_buffer::message::BufferMessage;

use crate::{
    error::AppError,
    event::{Emitter, Message},
    model::{DirectoryBufferState, Model, WindowType},
    open,
    task::Task,
    terminal::TerminalWrapper,
    update::{self, viewport},
};

#[derive(Debug)]
pub enum Action {
    EmitMessages(Vec<Message>),
    Load(WindowType, PathBuf, Option<String>),
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

pub struct ExecResult {
    pub result: ActionResult,
    pub remaining_actions: Vec<Action>,
}

#[tracing::instrument(skip(model, emitter, terminal, actions))]
pub async fn preview(
    model: &mut Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: Vec<Action>,
) -> Result<ExecResult, AppError> {
    execute(true, model, emitter, terminal, actions).await
}

#[tracing::instrument(skip(model, emitter, terminal, actions))]
pub async fn postview(
    model: &mut Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: Vec<Action>,
) -> Result<ExecResult, AppError> {
    execute(false, model, emitter, terminal, actions).await
}

fn is_preview_action(action: &Action) -> bool {
    match action {
        Action::Load(_, _, _) | Action::Open(_) | Action::Resize(_, _) | Action::Task(_) => true,

        Action::EmitMessages(_)
        | Action::ModeChanged
        | Action::Quit(_)
        | Action::UnwatchPath(_)
        | Action::WatchPath(_) => false,
    }
}

async fn execute(
    is_preview: bool,
    model: &mut Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: Vec<Action>,
) -> Result<ExecResult, AppError> {
    let result = if is_preview && contains_emit(&actions) {
        ActionResult::SkipRender
    } else if !is_preview && contains_quit(&actions) {
        ActionResult::Quit
    } else {
        ActionResult::Normal
    };

    let mut remaining_actions = vec![];
    for action in actions.into_iter() {
        if is_preview != is_preview_action(&action) {
            remaining_actions.push(action);
            continue;
        }

        tracing::debug!("handling action: {:?}", action);

        match action {
            Action::EmitMessages(messages) => {
                emitter.run(Task::EmitMessages(messages));
            }
            Action::Load(window_type, path, selection) => {
                match window_type {
                    WindowType::Current => {
                        model.files.current.state = DirectoryBufferState::Loading;
                        model.files.current.path = path.clone();

                        yeet_buffer::update::update_buffer(
                            &model.mode,
                            &mut model.files.current.buffer,
                            &BufferMessage::SetContent(Vec::new()),
                        );

                        viewport::set_viewport_dimensions(
                            &mut model.files.current.buffer.view_port,
                            &model.layout.current,
                        );

                        yeet_buffer::update::update_buffer(
                            &model.mode,
                            &mut model.files.current.buffer,
                            &BufferMessage::ResetCursor,
                        );

                        emitter.run(Task::EnumerateDirectory(path, selection.clone()));
                    }
                    WindowType::Parent | WindowType::Preview => {
                        update::set_buffer(&window_type, model, path.as_path(), vec![]);

                        if path.is_dir() {
                            emitter.run(Task::EnumerateDirectory(path.clone(), selection.clone()));
                        } else {
                            emitter.run(Task::LoadPreview(path.clone(), model.layout.preview));
                        }
                    }
                };
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

                // TODO: remove flickering (alternate screen leave and cli started)
                open::path(&path).await?;

                emitter.resume();
                terminal.resume()?;
            }
            Action::Quit(stdout_result) => {
                if let Some(stdout_result) = stdout_result {
                    if let Some(target) = &model.settings.selection_to_file_on_open {
                        emitter.run(Task::SaveSelection(target.clone(), stdout_result.clone()));
                    }

                    if model.settings.selection_to_stdout_on_open {
                        stdout().lock().write_all(stdout_result.as_bytes())?;
                    }
                }
                emitter.run(Task::SaveHistory(model.history.clone()));
                emitter.run(Task::SaveMarks(model.marks.clone()));
                emitter.run(Task::SaveQuickFix(model.qfix.clone()));
            }
            Action::Resize(x, y) => {
                terminal.resize(x, y)?;

                if let Some(path) = &model.files.preview.resolve_path() {
                    emitter.run(Task::LoadPreview(path.to_path_buf(), model.layout.preview));
                }
            }
            Action::Task(task) => emitter.run(task),
            Action::UnwatchPath(path) => {
                if path == PathBuf::default() {
                    continue;
                }

                emitter.abort(&Task::EnumerateDirectory(path.clone(), None));

                if let Err(error) = emitter.unwatch(path.as_path()) {
                    tracing::debug!("emitting unwatch path failed: {:?}", error);
                }
            }
            Action::WatchPath(path) => {
                if path == PathBuf::default() {
                    continue;
                }

                if let Err(error) = emitter.watch(path.as_path()) {
                    tracing::error!("emitting watch path failed: {:?}", error);
                }
            }
        }
    }

    Ok(ExecResult {
        result,
        remaining_actions,
    })
}

fn contains_emit(actions: &[Action]) -> bool {
    actions.iter().any(|a| matches!(a, Action::EmitMessages(_)))
}

fn contains_quit(actions: &[Action]) -> bool {
    actions.iter().any(|a| matches!(a, Action::Quit(_)))
}
