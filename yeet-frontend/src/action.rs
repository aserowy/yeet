use std::{
    io::{stdout, Write},
    path::PathBuf,
    time::Duration,
};

use yeet_keymap::message::Mode;

use crate::{error::AppError, event::Emitter, open, task::Task, terminal::TerminalWrapper};

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    PreView(PreView),
    PostView(PostView),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreView {
    Open(PathBuf),
    Resize(u16, u16),
    SleepBeforeRender,
    UnwatchPath(PathBuf),
    WatchPath(PathBuf),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PostView {
    ModeChanged(Mode),
    Quit(Option<String>),
    Task(Task),
}

pub async fn execute_pre_view(
    actions: &Vec<Action>,
    emitter: &mut Emitter,
    terminal: &mut TerminalWrapper,
) -> Result<(), AppError> {
    for action in actions {
        match action {
            Action::PreView(pre) => match pre {
                PreView::Open(path) => {
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
                }
                PreView::Resize(x, y) => {
                    terminal.resize(*x, *y)?;
                }
                PreView::SleepBeforeRender => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                PreView::UnwatchPath(p) => {
                    if p == &PathBuf::default() {
                        continue;
                    }

                    emitter.abort(&Task::EnumerateDirectory(p.clone()));

                    if let Err(_error) = emitter.unwatch(p.as_path()) {
                        // TODO: log error
                    }
                }
                PreView::WatchPath(p) => {
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
            },
            _ => {}
        }
    }
    Ok(())
}

pub async fn execute_post_view(
    actions: &Vec<Action>,
    emitter: &mut Emitter,
) -> Result<bool, AppError> {
    for action in actions {
        match action {
            Action::PostView(post) => match post {
                PostView::ModeChanged(mode) => {
                    emitter.set_current_mode(mode.clone()).await;
                }
                PostView::Quit(stdout_result) => {
                    if let Some(stdout_result) = stdout_result {
                        stdout().lock().write_all(stdout_result.as_bytes())?;
                    }
                    return Ok(false);
                }
                PostView::Task(task) => emitter.run(task.clone()),
            },
            _ => {}
        }
    }
    Ok(true)
}
