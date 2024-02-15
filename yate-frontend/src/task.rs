use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use tokio::{
    fs,
    sync::mpsc::Sender,
    task::{AbortHandle, JoinSet},
};

use crate::{error::AppError, event::RenderAction, model::history};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Task {
    AddPath(PathBuf),
    DeletePath(PathBuf),
    EnumerateDirectory(PathBuf),
    LoadPreview(PathBuf),
    OptimizeHistory,
    RenamePath(PathBuf, PathBuf),
}

pub struct TaskManager {
    abort_handles: HashMap<Task, AbortHandle>,
    sender: Sender<RenderAction>,
    tasks: JoinSet<Result<(), AppError>>,
}

impl TaskManager {
    pub fn new(sender: Sender<RenderAction>) -> Self {
        Self {
            abort_handles: HashMap::new(),
            sender,
            tasks: JoinSet::new(),
        }
    }

    pub fn abort(&mut self, task: &Task) {
        if let Some(abort_handle) = self.abort_handles.remove(task) {
            abort_handle.abort();
        }
    }

    pub async fn finishing(&mut self) -> Result<(), AppError> {
        let mut errors = Vec::new();
        while let Some(task) = self.tasks.join_next().await {
            match task {
                Ok(Ok(())) => (),
                // TODO: log error
                Ok(Err(error)) => errors.push(error),
                // TODO: log error
                Err(_) => (),
            };
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Aggregate(errors))
        }
    }

    // TODO: if error occurs, enable handling in model with RenderAction + sender
    pub fn run(&mut self, task: Task) {
        let abort_handle = match task.clone() {
            Task::AddPath(path) => self.tasks.spawn(async move {
                if path.exists() {
                    return Err(AppError::InvalidTargetPath);
                }

                if let Some(path_str) = path.to_str() {
                    if path_str.ends_with('/') {
                        fs::create_dir_all(path).await?;
                    } else {
                        let parent = match Path::new(&path).parent() {
                            Some(path) => path,
                            None => return Err(AppError::InvalidTargetPath),
                        };

                        fs::create_dir_all(parent).await?;
                        fs::write(path, "").await?;
                    }
                }

                Ok(())
            }),
            Task::DeletePath(path) => self.tasks.spawn(async move {
                if !path.exists() {
                    return Err(AppError::InvalidTargetPath);
                }

                if path.is_file() {
                    fs::remove_file(&path).await?;
                } else if path.is_dir() {
                    fs::remove_dir_all(&path).await?;
                };

                Ok(())
            }),
            Task::EnumerateDirectory(path) => {
                let internal_sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if !path.exists() {
                        return Err(AppError::InvalidTargetPath);
                    }

                    let read_dir = tokio::fs::read_dir(path.clone()).await;
                    let mut cache = Vec::new();
                    match read_dir {
                        Ok(mut rd) => {
                            let mut cache_size = 100;
                            let max_cache_size = 6_400;
                            while let Some(entry) = rd.next_entry().await? {
                                if cache.len() >= cache_size {
                                    cache.push(entry.path());

                                    // TODO: introduce custom message for this that contains all entries and frontload
                                    // bufferline creation, sorting and filtering to enable simple content replace
                                    let _ = internal_sender
                                        .send(RenderAction::PathsAdded(cache.drain(..).collect()))
                                        .await;

                                    if cache_size < max_cache_size {
                                        cache_size = cache_size * 2;
                                    }
                                } else {
                                    cache.push(entry.path());
                                }
                            }

                            let _ = internal_sender.send(RenderAction::PathsAdded(cache)).await;
                            let _ = internal_sender
                                .send(RenderAction::PathEnumerationFinished(path))
                                .await;

                            Ok(())
                        }
                        Err(error) => Err(AppError::FileOperationFailed(error)),
                    }
                })
            }
            Task::LoadPreview(path) => {
                let internal_sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Some(kind) = infer::get_from_path(path.clone())? {
                        // TODO: add preview for images here
                        if !kind.mime_type().starts_with("text") {
                            return Err(AppError::InvalidTargetPath);
                        }
                    }

                    let content = fs::read_to_string(path.clone()).await?;
                    let _ = internal_sender
                        .send(RenderAction::PreviewLoaded(
                            path.clone(),
                            content.lines().map(|s| s.to_string()).collect(),
                        ))
                        .await;

                    Ok(())
                })
            }
            Task::OptimizeHistory => self.tasks.spawn(async move {
                history::cache::optimize()?;

                Ok(())
            }),
            Task::RenamePath(old, new) => self.tasks.spawn(async move {
                if !old.exists() {
                    return Err(AppError::InvalidTargetPath);
                }

                fs::rename(old, new).await?;

                Ok(())
            }),
        };

        if let Some(handle) = self.abort_handles.insert(task, abort_handle) {
            handle.abort();
        }
    }
}
