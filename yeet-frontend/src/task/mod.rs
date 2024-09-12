use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use mime_guess::mime;
use ratatui::layout::Rect;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
use tokio::{
    fs,
    sync::{mpsc::Sender, Mutex},
    task::{AbortHandle, JoinSet},
};
use yeet_keymap::{
    conversion,
    message::{ContentKind, Envelope, KeySequence, Message, MessageSource},
    MessageResolver,
};

use crate::{
    error::AppError,
    init::{
        history::{optimize_history_file, save_history_to_file},
        junkyard::{cache_and_compress, compress, delete, restore},
        mark::{load_marks_from_file, save_marks_to_file},
        qfix::save_qfix_to_files,
    },
    model::{history::History, junkyard::FileEntry, mark::Marks, qfix::QuickFix},
};

mod image;
mod syntax;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Task {
    AddPath(PathBuf),
    CopyPath(PathBuf, PathBuf),
    DeleteMarks(Vec<char>),
    DeletePath(PathBuf),
    DeleteJunkYardEntry(FileEntry),
    EmitMessages(Vec<Message>),
    EnumerateDirectory(PathBuf, Option<String>),
    LoadPreview(PathBuf, Rect),
    RenamePath(PathBuf, PathBuf),
    RestorePath(FileEntry, PathBuf),
    SaveHistory(History),
    SaveMarks(Marks),
    SaveQuickFix(QuickFix),
    SaveSelection(PathBuf, String),
    TrashPath(FileEntry),
    YankPath(FileEntry),
}

pub struct TaskManager {
    abort_handles: Vec<(Task, AbortHandle)>,
    highlighter: Arc<Mutex<(SyntaxSet, ThemeSet)>>,
    resolver: Arc<Mutex<MessageResolver>>,
    sender: Sender<Envelope>,
    tasks: JoinSet<Result<(), AppError>>,
}

// TODO: harmonize error handling and tracing
// TODO: look into structured async to prevent arc mutexes all together
impl TaskManager {
    pub fn new(sender: Sender<Envelope>, resolver: Arc<Mutex<MessageResolver>>) -> Self {
        Self {
            abort_handles: Vec::new(),
            highlighter: Arc::new(Mutex::new((
                SyntaxSet::load_defaults_newlines(),
                ThemeSet::load_defaults(),
            ))),
            resolver,
            sender,
            tasks: JoinSet::new(),
        }
    }

    pub fn abort(&mut self, task: &Task) {
        if let Some(index) = self.get_abort_position(task) {
            let (_, abort_handle) = self.abort_handles.remove(index);
            abort_handle.abort();
        }
    }

    fn get_abort_position(&self, task: &Task) -> Option<usize> {
        match task {
            Task::EnumerateDirectory(path, _) => self
                .abort_handles
                .iter()
                .position(|(t, _)| matches!(t, Task::EnumerateDirectory(p, _) if p == path)),

            task => self.abort_handles.iter().position(|(t, _)| t == task),
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

    #[tracing::instrument(skip(self))]
    pub fn run(&mut self, task: Task) {
        tracing::trace!("running task: {:?}", task);

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
            Task::CopyPath(source, target) => self.tasks.spawn(async move {
                if !source.exists() || target.exists() {
                    return Err(AppError::InvalidTargetPath);
                }

                fs::copy(source, target).await?;

                Ok(())
            }),
            Task::DeleteMarks(marks) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    tracing::trace!("saving marks");

                    let mut current = Marks::default();
                    if let Err(err) = load_marks_from_file(&mut current) {
                        emit_error(&sender, err).await;
                        return Ok(());
                    }

                    for mark in marks {
                        current.entries.remove(&mark);
                    }

                    if let Err(error) = save_marks_to_file(&current) {
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
            Task::DeleteJunkYardEntry(entry) => self.tasks.spawn(async move {
                if let Err(error) = delete(entry).await {
                    tracing::error!("deleting junk yard entry failed: {:?}", error);
                }
                Ok(())
            }),
            Task::EmitMessages(messages) => {
                let sender = self.sender.clone();
                let resolver = self.resolver.clone();
                self.tasks.spawn(async move {
                    let (execute, messages): (Vec<_>, Vec<_>) = messages
                        .into_iter()
                        .partition(|m| matches!(m, Message::ExecuteKeySequence(_)));

                    let mut envelope = to_envelope(messages);

                    let sequence = execute
                        .iter()
                        .map(|m| match m {
                            Message::ExecuteKeySequence(sequence) => sequence.clone(),
                            _ => unreachable!(),
                        })
                        .collect::<Vec<_>>()
                        .join("");

                    let keys = conversion::from_keycode_string(&sequence);
                    let mut resolver = resolver.lock().await;
                    if let Some(resolved) = resolver.add_keys(keys) {
                        envelope.messages.extend(resolved.messages);
                        envelope.sequence = resolved.sequence;
                    }

                    // NOTE: important to prevent deadlock for queue size of one
                    drop(resolver);

                    if let Err(error) = sender.send(envelope).await {
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
                                        .send(to_envelope(vec![Message::EnumerationChanged(
                                            path.clone(),
                                            cache.clone(),
                                            selection.clone(),
                                        )]))
                                        .await;

                                    cache_size *= 2;
                                }
                            }

                            let _ = internal_sender
                                .send(to_envelope(vec![
                                    Message::EnumerationChanged(
                                        path.clone(),
                                        cache.clone(),
                                        selection.clone(),
                                    ),
                                    Message::EnumerationFinished(path, selection),
                                ]))
                                .await;

                            Ok(())
                        }
                        Err(error) => Err(AppError::FileOperationFailed(error)),
                    }
                })
            }
            Task::LoadPreview(path, rect) => {
                let sender = self.sender.clone();
                let highlighter = Arc::clone(&self.highlighter);
                self.tasks.spawn(async move {
                    let highlighter = highlighter.lock().await;
                    let (syntaxes, theme_set) = (&highlighter.0, &highlighter.1);
                    let theme = &theme_set.themes["base16-eighties.dark"];

                    let content = if let Some(mime) = mime_guess::from_path(path.clone()).first() {
                        match (mime.type_(), mime.subtype()) {
                            (mime::IMAGE, _) => match image::load(&path, &rect).await {
                                Some(content) => content,
                                None => "".to_string(),
                            },
                            (mime::TEXT, _) | (mime::APPLICATION, mime::JSON) => {
                                match syntax::highlight(syntaxes, theme, &path).await {
                                    Some(content) => content,
                                    None => "".to_string(),
                                }
                            }
                            _ => {
                                tracing::debug!("no preview specified for mime: {:?}", mime);
                                "".to_string()
                            }
                        }
                    } else {
                        tracing::debug!("unable to resolve kind for: {:?}", path);

                        match syntax::highlight(syntaxes, theme, &path).await {
                            Some(content) => content,
                            None => "".to_string(),
                        }
                    };

                    let result = sender
                        .send(to_envelope(vec![Message::PreviewLoaded(
                            path.clone(),
                            content.lines().map(|s| s.to_string()).collect(),
                        )]))
                        .await;

                    if let Err(error) = result {
                        emit_error(&sender, AppError::ActionSendFailed(error)).await;
                    }

                    Ok(())
                })
            }
            Task::RenamePath(old, new) => self.tasks.spawn(async move {
                if !old.exists() || new.exists() {
                    return Err(AppError::InvalidTargetPath);
                }

                fs::rename(old, new).await?;

                Ok(())
            }),
            Task::RestorePath(entry, path) => self.tasks.spawn(async move {
                restore(entry, path)?;
                Ok(())
            }),
            Task::SaveHistory(history) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Err(error) = save_history_to_file(&history) {
                        emit_error(&sender, error).await;
                    }
                    optimize_history_file()?;

                    Ok(())
                })
            }
            Task::SaveMarks(marks) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    tracing::trace!("saving marks");

                    if let Err(error) = save_marks_to_file(&marks) {
                        emit_error(&sender, error).await;
                    }

                    Ok(())
                })
            }
            Task::SaveQuickFix(qfix) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    tracing::trace!("saving qfix");

                    if let Err(error) = save_qfix_to_files(&qfix) {
                        emit_error(&sender, error).await;
                    }

                    Ok(())
                })
            }
            Task::SaveSelection(target, selection) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    tracing::trace!("saving selection to file {}", target.to_string_lossy());

                    if let Err(error) = tokio::fs::write(target, selection).await {
                        emit_error(&sender, AppError::FileOperationFailed(error)).await;
                    }

                    Ok(())
                })
            }
            Task::TrashPath(entry) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Err(error) = cache_and_compress(entry).await {
                        emit_error(&sender, error).await;
                    }

                    Ok(())
                })
            }
            Task::YankPath(entry) => {
                let sender = self.sender.clone();
                self.tasks.spawn(async move {
                    if let Err(error) = compress(entry).await {
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

async fn emit_error(sender: &Sender<Envelope>, error: AppError) {
    tracing::error!("task failed: {:?}", error);

    let error = format!("Error: {:?}", error);
    let _ = sender.send(to_envelope(vec![Message::Error(error)])).await;
}

fn to_envelope(messages: Vec<Message>) -> Envelope {
    Envelope {
        messages,
        sequence: KeySequence::None,
        source: MessageSource::Task,
    }
}

fn should_abort_on_finish(task: Task) -> bool {
    match task {
        Task::EmitMessages(_) | Task::EnumerateDirectory(_, _) | Task::LoadPreview(_, _) => true,

        Task::AddPath(_)
        | Task::CopyPath(_, _)
        | Task::DeleteMarks(_)
        | Task::DeletePath(_)
        | Task::DeleteJunkYardEntry(_)
        | Task::RenamePath(_, _)
        | Task::RestorePath(_, _)
        | Task::SaveHistory(_)
        | Task::SaveMarks(_)
        | Task::SaveQuickFix(_)
        | Task::SaveSelection(_, _)
        | Task::TrashPath(_)
        | Task::YankPath(_) => false,
    }
}
