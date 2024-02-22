use std::{
    io::{stdout, Write},
    path::PathBuf,
    time::Duration,
};

use yeet_keymap::message::Mode;

use crate::{
    error::AppError, event::Emitter, model::Model, open, task::Task, terminal::TerminalWrapper,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    PreView(PreView),
    PostView(PostView),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreView {
    Open(PathBuf),
    Resize(u16, u16),
    SkipRender,
    SleepBeforeRender,
    UnwatchPath(PathBuf),
    WatchPath(PathBuf),
}

pub async fn execute_pre_view(
    actions: &Vec<Action>,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
) -> Result<bool, AppError> {
    let mut result = true;
    for action in actions {
        if let Action::PreView(pre) = action {
            match pre {
                PreView::Open(path) => {
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
                    open::path(path).await?;

                    emitter.resume();
                    terminal.resume()?;
                }
                PreView::Resize(x, y) => {
                    terminal.resize(*x, *y)?;
                }
                PreView::SkipRender => result = false,
                PreView::SleepBeforeRender => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                PreView::UnwatchPath(path) => {
                    if path == &PathBuf::default() {
                        continue;
                    }

                    emitter.abort(&Task::EnumerateDirectory(path.clone()));

                    if let Err(_error) = emitter.unwatch(path.as_path()) {
                        // TODO: log error
                    }
                }
                PreView::WatchPath(path) => {
                    if path == &PathBuf::default() {
                        continue;
                    }

                    if path.is_dir() {
                        emitter.run(Task::EnumerateDirectory(path.clone()));
                    } else {
                        emitter.run(Task::LoadPreview(path.clone()));
                    }

                    if let Err(_error) = emitter.watch(path.as_path()) {
                        // TODO: log error
                    }
                }
            }
        }
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq)]
pub enum PostView {
    ModeChanged(Mode),
    Quit(Option<String>),
    Task(Task),
}

pub async fn execute_post_view(
    actions: &Vec<Action>,
    emitter: &mut Emitter,
    model: &Model,
) -> Result<bool, AppError> {
    for action in actions {
        if let Action::PostView(post) = action {
            match post {
                PostView::ModeChanged(mode) => {
                    emitter.set_current_mode(mode.clone()).await;
                }
                PostView::Quit(stdout_result) => {
                    if let Some(stdout_result) = stdout_result {
                        stdout().lock().write_all(stdout_result.as_bytes())?;
                    }
                    emitter.run(Task::SaveHistory(model.history.clone()));

                    return Ok(false);
                }
                PostView::Task(task) => emitter.run(task.clone()),
            }
        }
    }
    Ok(true)
}
