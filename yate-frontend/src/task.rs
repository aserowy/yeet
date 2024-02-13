use std::path::{Path, PathBuf};

use tokio::{fs, sync::mpsc::UnboundedSender, task::JoinSet};

use crate::{error::AppError, event::RenderAction, model::history};

#[derive(Clone, Debug, PartialEq)]
pub enum Task {
    AddPath(PathBuf),
    DeletePath(PathBuf),
    EnumerateDirectory(PathBuf),
    OptimizeHistory,
    RenamePath(PathBuf, PathBuf),
}

pub struct TaskManager {
    sender: UnboundedSender<RenderAction>,
    tasks: JoinSet<Result<(), AppError>>,
}

impl TaskManager {
    pub fn new(sender: UnboundedSender<RenderAction>) -> Self {
        Self {
            sender,
            tasks: JoinSet::new(),
        }
    }

    // TODO: if error occurs, enable handling in model with RenderAction + sender
    pub fn run(&mut self, task: Task) {
        match task {
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
            // TODO: batch dirs and send message batches with 500 entries
            Task::EnumerateDirectory(path) => {
                let internal_sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if !path.exists() {
                        return Err(AppError::InvalidTargetPath);
                    }

                    let read_dir = tokio::fs::read_dir(path).await;
                    match read_dir {
                        Ok(mut rd) => {
                            while let Some(entry) = rd.next_entry().await? {
                                internal_sender
                                    .send(RenderAction::PathAdded(entry.path()))
                                    .unwrap();
                            }

                            Ok(())
                        }
                        Err(error) => Err(AppError::FileOperationFailed(error)),
                    }
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
}
