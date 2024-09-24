use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use ratatui::layout::Rect;
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
use tokio::{
    fs,
    sync::{mpsc::Sender, Mutex},
    task::{AbortHandle, JoinSet},
};
use yeet_keymap::{
    conversion,
    message::{KeySequence, KeymapMessage},
    MessageResolver,
};

use crate::{
    error::AppError,
    event::{ContentKind, Envelope, Message, MessageSource},
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

#[derive(Debug)]
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

impl Task {
    pub fn to_identifier_string(&self) -> String {
        match self {
            Task::AddPath(path) => format!("AddPath({:?})", path),
            Task::CopyPath(src, dst) => format!("CopyPath({:?}, {:?})", src, dst),
            Task::DeleteMarks(marks) => format!("DeleteMarks({:?})", marks),
            Task::DeletePath(path) => format!("DeletePath({:?})", path),
            Task::DeleteJunkYardEntry(entry) => format!("DeleteJunkYardEntry({:?})", entry),
            Task::EmitMessages(_) => "EmitMessages".to_string(),
            Task::EnumerateDirectory(path, _) => {
                format!("EnumerateDirectory({:?}, _)", path)
            }
            Task::LoadPreview(path, rect) => format!("LoadPreview({:?}, {})", path, rect),
            Task::RenamePath(old, new) => {
                format!("RenamePath({:?}, {:?})", old, new)
            }
            Task::RestorePath(entry, path) => {
                format!("RestorePath({:?}, {:?})", entry, path)
            }
            Task::SaveHistory(_) => "SaveHistory".to_string(),
            Task::SaveMarks(_) => "SaveMarks".to_string(),
            Task::SaveQuickFix(_) => "SaveQuickFix".to_string(),
            Task::SaveSelection(path, _) => format!("SaveSelection({:?}, _)", path),
            Task::TrashPath(entry) => format!("TrashPath({:?})", entry),
            Task::YankPath(entry) => format!("YankPath({:?})", entry),
        }
    }
}

impl Eq for Task {}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Task::AddPath(p1), Task::AddPath(p2)) => p1 == p2,
            (Task::CopyPath(s1, t1), Task::CopyPath(s2, t2)) => s1 == s2 && t1 == t2,
            (Task::DeleteMarks(m1), Task::DeleteMarks(m2)) => m1 == m2,
            (Task::DeletePath(p1), Task::DeletePath(p2)) => p1 == p2,
            (Task::DeleteJunkYardEntry(e1), Task::DeleteJunkYardEntry(e2)) => e1 == e2,
            (Task::EnumerateDirectory(p1, s1), Task::EnumerateDirectory(p2, s2)) => {
                p1 == p2 && s1 == s2
            }
            (Task::LoadPreview(p1, r1), Task::LoadPreview(p2, r2)) => p1 == p2 && r1 == r2,
            (Task::RenamePath(o1, n1), Task::RenamePath(o2, n2)) => o1 == o2 && n1 == n2,
            (Task::RestorePath(e1, p1), Task::RestorePath(e2, p2)) => e1 == e2 && p1 == p2,
            (Task::SaveHistory(h1), Task::SaveHistory(h2)) => h1 == h2,
            (Task::SaveMarks(m1), Task::SaveMarks(m2)) => m1 == m2,
            (Task::SaveQuickFix(q1), Task::SaveQuickFix(q2)) => q1 == q2,
            (Task::SaveSelection(p1, s1), Task::SaveSelection(p2, s2)) => p1 == p2 && s1 == s2,
            (Task::TrashPath(e1), Task::TrashPath(e2)) => e1 == e2,
            (Task::YankPath(e1), Task::YankPath(e2)) => e1 == e2,
            _ => false,
        }
    }
}

pub struct TaskManager {
    abort_handles: HashMap<String, AbortHandle>,
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
            abort_handles: HashMap::new(),
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
        let task_identifier = task.to_identifier_string();
        if let Some(abort_handle) = self.abort_handles.remove(&task_identifier) {
            abort_handle.abort();
        }
    }

    // TODO: result should handle shell code on exit
    pub async fn finishing(&mut self) -> Result<(), AppError> {
        let mut errors = Vec::new();
        for (_identifier, abort_handle) in self.abort_handles.drain() {
            // TODO: readd function
            // if should_abort_on_finish(task) {
            abort_handle.abort();
            // }
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

        let task_identifier = task.to_identifier_string();
        let abort_handle = match task {
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
                    let (execute, messages): (Vec<_>, Vec<_>) =
                        messages.into_iter().partition(|m| {
                            matches!(m, Message::Keymap(KeymapMessage::ExecuteKeySequence(_)))
                        });

                    let mut envelope = to_envelope(messages);

                    let sequence = execute
                        .iter()
                        .map(|m| match m {
                            Message::Keymap(KeymapMessage::ExecuteKeySequence(sequence)) => {
                                sequence.clone()
                            }
                            _ => unreachable!(),
                        })
                        .collect::<Vec<_>>()
                        .join("");

                    let keys = conversion::from_keycode_string(&sequence);
                    let mut resolver = resolver.lock().await;
                    if let Some(resolved) = resolver.add_keys(keys) {
                        let messages: Vec<_> =
                            resolved.0.into_iter().map(|m| Message::Keymap(m)).collect();
                        envelope.messages.extend(messages);
                        envelope.sequence = resolved.1;
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

                    let mime = if let Some(mime) = infer::get_from_path(&path)? {
                        let kind = mime.mime_type().split('/').collect::<Vec<_>>();
                        if kind.len() != 2 {
                            return Err(AppError::InvalidMimeType);
                        }
                        Some(kind[0].to_ascii_lowercase())
                    } else {
                        None
                    };

                    let content = match mime.as_deref() {
                        Some("image") => image::load(&path, &rect).await,
                        _ => syntax::highlight(syntaxes, theme, &path).await,
                    };

                    let result = sender
                        .send(to_envelope(vec![Message::PreviewLoaded(content)]))
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

        if let Some(abort_handle) = self.abort_handles.insert(task_identifier, abort_handle) {
            abort_handle.abort();
        }
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
