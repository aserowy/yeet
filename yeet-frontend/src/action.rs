use std::{
    io::{stdout, Write},
    path::PathBuf,
};

use crate::{
    error::AppError,
    event::{Emitter, Message},
    model::{Model, WindowType},
    open,
    task::Task,
    terminal::TerminalWrapper,
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

#[tracing::instrument(skip(model, emitter, terminal, actions))]
pub async fn exec_preview(
    model: &mut Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: Vec<Action>,
) -> Result<(Vec<Action>, ActionResult), AppError> {
    execute(true, model, emitter, terminal, actions).await
}

#[tracing::instrument(skip(model, emitter, terminal, actions))]
pub async fn exec_postview(
    model: &mut Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: Vec<Action>,
) -> Result<(Vec<Action>, ActionResult), AppError> {
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
    model: &mut Model,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
    actions: Vec<Action>,
) -> Result<(Vec<Action>, ActionResult), AppError> {
    let result = if is_preview && contains_emit(&actions) {
        ActionResult::SkipRender
    } else if !is_preview && contains_quit(&actions) {
        ActionResult::Quit
    } else {
        ActionResult::Normal
    };

    let mut not_handled_actions = vec![];
    for action in actions.into_iter() {
        if is_preview != is_preview_action(&action) {
            not_handled_actions.push(action);
            continue;
        }

        tracing::debug!("handling action: {:?}", action);

        match action {
            Action::EmitMessages(messages) => {
                emitter.run(Task::EmitMessages(messages));
            }
            Action::Load(window_type, path, selection) => {
                // set from outside which buffer
                // if normal clear buffer lines, set state to Load
                // model.files.preview.buffer.lines.clear();
                // model.files.preview.state = DirectoryBufferState::Loading;
                // set_viewport_dimensions(&mut buffer.view_port, layout);
                // update_buffer(&model.mode, buffer, &BufferMessage::ResetCursor);
                // reset viewport/cursor
                // if preview set state to load
                if path.is_dir() {
                    emitter.run(Task::EnumerateDirectory(path.clone(), selection.clone()));
                } else {
                    emitter.run(Task::LoadPreview(path.clone(), model.layout.preview));
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

    Ok((not_handled_actions, result))
}

fn contains_emit(actions: &[Action]) -> bool {
    actions.iter().any(|a| matches!(a, Action::EmitMessages(_)))
}

fn contains_quit(actions: &[Action]) -> bool {
    actions.iter().any(|a| matches!(a, Action::Quit(_)))
}
