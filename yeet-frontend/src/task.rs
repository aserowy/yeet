use std::path::{Path, PathBuf};

use tokio::{
    fs,
    sync::mpsc::Sender,
    task::{AbortHandle, JoinSet},
};
use yeet_keymap::message::{ContentKind, Message};

use crate::{
    error::AppError,
    model::{
        history::{self, History},
        mark::{self, Marks},
        qfix::{self, QuickFix},
        register::{self, FileEntry},
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Task {
    AddPath(PathBuf),
    DeleteMarks(Vec<char>),
    DeletePath(PathBuf),
    DeleteJunkYardEntry(FileEntry),
    EmitMessages(Vec<Message>),
    EnumerateDirectory(PathBuf, Option<String>),
    LoadPreview(PathBuf),
    OptimizeHistory,
    RenamePath(PathBuf, PathBuf),
    RestorePath(FileEntry, PathBuf),
    SaveHistory(History),
    SaveMarks(Marks),
    SaveQuickFix(QuickFix),
    TrashPath(FileEntry),
    YankPath(FileEntry),
}

pub struct TaskManager {
    abort_handles: Vec<(Task, AbortHandle)>,
    sender: Sender<Vec<Message>>,
    tasks: JoinSet<Result<(), AppError>>,
}

// TODO: harmonize error handling and tracing
impl TaskManager {
    pub fn new(sender: Sender<Vec<Message>>) -> Self {
        Self {
            abort_handles: Vec::new(),
            sender,
            tasks: JoinSet::new(),
        }
    }

    pub fn abort(&mut self, task: &Task) {
        if let Some(index) = self.abort_handles.iter().position(|(t, _)| t == task) {
            let (_, abort_handle) = self.abort_handles.remove(index);
            abort_handle.abort();
        }
    }

    // TODO: result should handle shell code on exit
    pub async fn finishing(&mut self) -> Result<(), AppError> {
        let mut errors = Vec::new();
        for (task, abort_handle) in self.abort_handles.drain(..) {
            if should_abort_on_finish(task) {
                abort_handle.abort();
            }
        }

        while let Some(task) = self.tasks.join_next().await {
            match task {
                Ok(Ok(())) => (),
                Ok(Err(error)) => {
                    tracing::error!("task result returned error: {:?}", error);
                    errors.push(error)
                }
                Err(error) => {
                    tracing::error!("task failed: {:?}", error);
                }
            };
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Aggregate(errors))
        }
    }

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
            Task::DeleteMarks(marks) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    tracing::trace!("saving marks");

                    let mut current = Marks::default();
                    if let Err(err) = mark::load(&mut current) {
                        emit_error(&sender, err).await;
                        return Ok(());
                    }

                    for mark in marks {
                        current.entries.remove(&mark);
                    }

                    if let Err(error) = mark::save(&current) {
                        emit_error(&sender, error).await;
                    }

                    Ok(())
                })
            }
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
            Task::DeleteJunkYardEntry(entry) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Err(error) = register::file::delete(entry).await {
                        emit_error(&sender, error).await;
                    }
                    Ok(())
                })
            }
            Task::EmitMessages(messages) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Err(error) = sender.send(messages).await {
                        emit_error(&sender, AppError::ActionSendFailed(error)).await;
                    }
                    Ok(())
                })
            }
            Task::EnumerateDirectory(path, selection) => {
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

                            let (is_selection, selection_path) = match &selection {
                                Some(selection) => {
                                    let path = path.join(selection);
                                    if path.exists() {
                                        let kind = if path.is_dir() {
                                            ContentKind::Directory
                                        } else {
                                            ContentKind::File
                                        };

                                        cache.push((kind, selection.clone()));

                                        (true, path)
                                    } else {
                                        (false, PathBuf::new())
                                    }
                                }
                                None => (false, PathBuf::new()),
                            };

                            while let Some(entry) = rd.next_entry().await? {
                                let kind = if entry.path().is_dir() {
                                    ContentKind::Directory
                                } else {
                                    ContentKind::File
                                };

                                let content = match entry.path().file_name() {
                                    Some(content) => content.to_str().unwrap_or("").to_string(),
                                    None => "".to_string(),
                                };

                                if !is_selection || entry.path() != selection_path {
                                    cache.push((kind, content));
                                }

                                if cache.len() >= cache_size {
                                    let _ = internal_sender
                                        .send(vec![Message::EnumerationChanged(
                                            path.clone(),
                                            cache.clone(),
                                            selection.clone(),
                                        )])
                                        .await;

                                    cache_size *= 2;
                                }
                            }

                            let _ = internal_sender
                                .send(vec![
                                    Message::EnumerationChanged(
                                        path.clone(),
                                        cache.clone(),
                                        selection.clone(),
                                    ),
                                    Message::EnumerationFinished(path),
                                ])
                                .await;

                            Ok(())
                        }
                        Err(error) => Err(AppError::FileOperationFailed(error)),
                    }
                })
            }
            Task::LoadPreview(path) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Some(kind) = infer::get_from_path(path.clone())? {
                        // TODO: add preview for images here
                        // TODO: add preview for archives here
                        if !kind.mime_type().starts_with("text") {
                            return Ok(());
                        }
                    }

                    let content = fs::read_to_string(path.clone()).await?;
                    let result = sender
                        .send(vec![Message::PreviewLoaded(
                            path.clone(),
                            content.lines().map(|s| s.to_string()).collect(),
                        )])
                        .await;

                    if let Err(error) = result {
                        emit_error(&sender, AppError::ActionSendFailed(error)).await;
                    }

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
            Task::RestorePath(entry, path) => self.tasks.spawn(async move {
                register::file::restore(entry, path)?;
                Ok(())
            }),
            Task::SaveHistory(history) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Err(error) = history::cache::save(&history) {
                        emit_error(&sender, error).await;
                    }
                    history::cache::optimize()?;

                    Ok(())
                })
            }
            Task::SaveMarks(marks) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    tracing::trace!("saving marks");

                    if let Err(error) = mark::save(&marks) {
                        emit_error(&sender, error).await;
                    }

                    Ok(())
                })
            }
            Task::SaveQuickFix(qfix) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    tracing::trace!("saving qfix");

                    if let Err(error) = qfix::save(&qfix) {
                        emit_error(&sender, error).await;
                    }

                    Ok(())
                })
            }
            Task::TrashPath(entry) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Err(error) = register::file::cache_and_compress(entry).await {
                        emit_error(&sender, error).await;
                    }

                    Ok(())
                })
            }
            Task::YankPath(entry) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Err(error) = register::file::compress(entry).await {
                        emit_error(&sender, error).await;
                    }

                    Ok(())
                })
            }
        };

        if let Some(index) = self.abort_handles.iter().position(|(t, _)| t == &task) {
            let (_, abort_handle) = self.abort_handles.remove(index);
            abort_handle.abort();
        }

        self.abort_handles.push((task, abort_handle));
    }
}

async fn emit_error(sender: &Sender<Vec<Message>>, error: AppError) {
    tracing::error!("task failed: {:?}", error);

    let error = format!("Error: {:?}", error);
    let _ = sender.send(vec![Message::Error(error)]).await;
}

fn should_abort_on_finish(task: Task) -> bool {
    match task {
        Task::EmitMessages(_) | Task::EnumerateDirectory(_, _) | Task::LoadPreview(_) => true,

        Task::AddPath(_)
        | Task::DeleteMarks(_)
        | Task::DeletePath(_)
        | Task::DeleteJunkYardEntry(_)
        | Task::OptimizeHistory
        | Task::RenamePath(_, _)
        | Task::RestorePath(_, _)
        | Task::SaveHistory(_)
        | Task::SaveMarks(_)
        | Task::SaveQuickFix(_)
        | Task::TrashPath(_)
        | Task::YankPath(_) => false,
    }
}
