use std::{
    fmt::{Debug, Display},
    path::{Path, PathBuf},
    sync::Arc,
};

use ratatui::layout::Rect;
use ratatui_image::picker::{Picker, ProtocolType};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};
use tokio::{
    fs,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
};
use tokio_util::sync::CancellationToken;
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
        junkyard::{self, cache_and_compress, compress, restore},
        mark::{load_marks_from_file, save_marks_to_file},
        qfix::save_qfix_to_files,
    },
    model::{history::History, junkyard::FileEntry, mark::Marks, qfix::QuickFix},
};

mod command;
mod image;
mod syntax;

pub enum Task {
    AddPath(PathBuf),
    CopyPath(PathBuf, PathBuf),
    DeleteMarks(Vec<char>),
    DeletePath(PathBuf),
    DeleteJunkYardEntry(FileEntry),
    EmitMessages(Vec<Message>),
    EnumerateDirectory(PathBuf, Option<String>),
    ExecuteFd(PathBuf, String),
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::AddPath(path) => write!(f, "AddPath({:?})", path),
            Task::CopyPath(src, dst) => write!(f, "CopyPath({:?}, {:?})", src, dst),
            Task::DeleteMarks(marks) => write!(f, "DeleteMarks({:?})", marks),
            Task::DeletePath(path) => write!(f, "DeletePath({:?})", path),
            Task::DeleteJunkYardEntry(entry) => write!(f, "DeleteJunkYardEntry({:?})", entry),
            Task::EmitMessages(_) => write!(f, "EmitMessages"),
            Task::EnumerateDirectory(path, _) => write!(f, "EnumerateDirectory({:?}, _)", path),
            Task::ExecuteFd(base, params) => write!(f, "ExecuteFd({:?}, {:?})", base, params),
            Task::LoadPreview(path, rect) => write!(f, "LoadPreview({:?}, {})", path, rect),
            Task::RenamePath(old, new) => write!(f, "RenamePath({:?}, {:?})", old, new),
            Task::RestorePath(entry, path) => write!(f, "RestorePath({:?}, {:?})", entry, path),
            Task::SaveHistory(_) => write!(f, "SaveHistory"),
            Task::SaveMarks(_) => write!(f, "SaveMarks"),
            Task::SaveQuickFix(_) => write!(f, "SaveQuickFix"),
            Task::SaveSelection(path, _) => write!(f, "SaveSelection({:?}, _)", path),
            Task::TrashPath(entry) => write!(f, "TrashPath({:?})", entry),
            Task::YankPath(entry) => write!(f, "YankPath({:?})", entry),
        }
    }
}

impl Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
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
    pub sender: mpsc::UnboundedSender<Task>,
}

// TODO: look into structured async to prevent arc mutexes all together
// TODO: harmonize error handling and tracing
impl TaskManager {
    pub fn new(
        sender: Sender<Envelope>,
        resolver: Arc<Mutex<MessageResolver>>,
        cancellation: CancellationToken,
    ) -> Self {
        let picker = resolve_picker();

        tracing::info!("image picker configured: {:?}", picker);

        let resolver = resolver.clone();
        let (task_sender, mut task_receiver) = mpsc::unbounded_channel::<Task>();
        tokio::spawn(async move {
            let highlighter = Arc::new(Mutex::new((
                SyntaxSet::load_defaults_newlines(),
                ThemeSet::load_defaults(),
            )));
            let picker = Arc::new(Mutex::new(picker));
            loop {
                let child_token = cancellation.child_token();
                tokio::select! {
                    _ = cancellation.cancelled() => break,
                    task = task_receiver.recv() => {
                        let task = match task {
                            Some(it) => it,
                            None => return,
                        };

                        tracing::debug!("handling task: {:?}", task.to_string());

                        let sender = sender.clone();
                        let resolver = resolver.clone();
                        let highlighter = highlighter.clone();
                        let picker = picker.clone();

                        tokio::spawn(async move {
                            let id = task.to_string();
                            send_task_started(&sender.clone(), id.as_str(), child_token.clone()).await;

                            if let Err(err) = run_task(
                                &sender.clone(),
                                resolver,
                                highlighter,
                                picker,
                                task,
                                child_token
                            ).await
                            {
                                tracing::error!("handling task failed: {:?}", err);
                            };

                            send_task_finished(&sender, id.as_str()).await;
                        });
                    }
                }
            }
        });

        Self {
            sender: task_sender,
        }
    }
}

#[cfg(target_os = "windows")]
fn resolve_picker() -> Option<(Picker, ProtocolType)> {
    // FIX: https://github.com/benjajaja/ratatui-image/issues/32
    // otherwise make fontsize configureable
    None
}

#[cfg(not(target_os = "windows"))]
fn resolve_picker() -> Option<(Picker, ProtocolType)> {
    Picker::from_termios()
        .ok()
        .and_then(|mut picker| Some((picker, picker.guess_protocol())))
}

async fn run_task(
    sender: &Sender<Envelope>,
    resolver: Arc<Mutex<MessageResolver>>,
    highlighter: Arc<Mutex<(SyntaxSet, ThemeSet)>>,
    picker: Arc<Mutex<Option<(Picker, ProtocolType)>>>,
    task: Task,
    cancellation: CancellationToken,
) -> Result<(), AppError> {
    match task {
        Task::AddPath(path) => {
            if path.exists() {
                return Err(AppError::InvalidTargetPath);
            }

            if let Some(path_str) = path.to_str() {
                if path_str.ends_with('/') {
                    fs::create_dir_all(path).await?;
                } else {
                    let parent = match Path::new(&path).parent() {
                        Some(path) => path,
                        None => {
                            return Err(AppError::InvalidTargetPath);
                        }
                    };

                    fs::create_dir_all(parent).await?;
                    fs::write(path, "").await?;
                }
            }
        }
        Task::CopyPath(source, target) => {
            if !source.exists() || target.exists() {
                return Err(AppError::InvalidTargetPath);
            }

            fs::copy(source, target).await?;
        }
        Task::DeleteMarks(marks) => {
            let mut current = Marks::default();
            if let Err(err) = load_marks_from_file(&mut current) {
                emit_error(&sender, err).await;
            } else {
                for mark in marks {
                    current.entries.remove(&mark);
                }

                if let Err(error) = save_marks_to_file(&current) {
                    emit_error(&sender, error).await;
                }
            }
        }
        Task::DeletePath(path) => {
            if !path.exists() {
                return Err(AppError::InvalidTargetPath);
            };

            if path.is_file() {
                fs::remove_file(&path).await?;
            } else if path.is_dir() {
                fs::remove_dir_all(&path).await?;
            }
        }
        Task::DeleteJunkYardEntry(entry) => {
            if let Err(error) = junkyard::delete(entry).await {
                tracing::error!("deleting junk yard entry failed: {:?}", error);
            }
        }
        Task::EmitMessages(messages) => {
            let (execute, messages): (Vec<_>, Vec<_>) = messages
                .into_iter()
                .partition(|m| matches!(m, Message::Keymap(KeymapMessage::ExecuteKeySequence(_))));

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
            if let Some(resolved) = resolver.lock().await.add_keys(keys) {
                let messages: Vec<_> = resolved.0.into_iter().map(Message::Keymap).collect();

                envelope.messages.extend(messages);
                envelope.sequence = resolved.1;
            }

            // NOTE: important to prevent deadlock for queue size of one
            drop(resolver);

            if let Err(error) = sender.send(envelope).await {
                emit_error(&sender, AppError::ActionSendFailed(error)).await;
            }
        }
        Task::EnumerateDirectory(path, selection) => {
            if !path.exists() {
                return Err(AppError::InvalidTargetPath);
            }

            let read_dir = fs::read_dir(path.clone()).await;
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
                                tracing::warn!("path does not exist: {:?}", path);
                                (false, PathBuf::new())
                            }
                        }
                        None => (false, PathBuf::new()),
                    };

                    while let Ok(Some(entry)) = rd.next_entry().await {
                        if cancellation.is_cancelled() {
                            break;
                        }

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
                            let _ = sender
                                .send(to_envelope(vec![Message::EnumerationChanged(
                                    path.clone(),
                                    cache.clone(),
                                    selection.clone(),
                                )]))
                                .await;

                            cache_size *= 2;
                        }
                    }

                    if cancellation.is_cancelled() {
                        return Ok(());
                    }

                    let _ = sender
                        .send(to_envelope(vec![
                            Message::EnumerationChanged(
                                path.clone(),
                                cache.clone(),
                                selection.clone(),
                            ),
                            Message::EnumerationFinished(path, selection),
                        ]))
                        .await;
                }
                Err(error) => {
                    return Err(AppError::FileOperationFailed(error));
                }
            }
        }
        Task::ExecuteFd(base, params) => match command::fd(base.as_path(), params).await {
            Ok(paths) => {
                let result = sender
                    .send(to_envelope(vec![Message::FdResult(paths)]))
                    .await;

                if let Err(error) = result {
                    tracing::error!("sending message failed: {:?}", error);
                }
            }
            Err(err) => {
                emit_error(&sender, err).await;
            }
        },
        Task::LoadPreview(path, rect) => {
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
                Some("image") => {
                    // NOTE: each time protocol gets used (even for debug) the picker
                    // forgets the given protocol type
                    let mut picker = picker.lock().await.and_then(|(mut picker, protocol)| {
                        picker.protocol_type = protocol;
                        Some(picker)
                    });

                    image::load(&mut picker, &path, &rect).await
                }
                _ => {
                    let highlighter = highlighter.lock().await;
                    let (syntaxes, theme_set) = (&highlighter.0, &highlighter.1);
                    let theme = &theme_set.themes["base16-eighties.dark"];

                    syntax::highlight(syntaxes, theme, &path).await
                }
            };

            let result = sender
                .send(to_envelope(vec![Message::PreviewLoaded(content)]))
                .await;

            if let Err(error) = result {
                tracing::error!("sending message failed: {:?}", error);
            }
        }
        Task::RenamePath(old, new) => {
            if !old.exists() || new.exists() {
                return Err(AppError::InvalidTargetPath);
            }

            fs::rename(old, new).await?;
        }
        Task::RestorePath(entry, path) => {
            restore(entry, path)?;
        }
        Task::SaveHistory(history) => {
            if let Err(error) = save_history_to_file(&history) {
                emit_error(&sender, error).await;
            }
            optimize_history_file()?;
        }
        Task::SaveMarks(marks) => {
            if let Err(error) = save_marks_to_file(&marks) {
                emit_error(&sender, error).await;
            }
        }
        Task::SaveQuickFix(qfix) => {
            if let Err(error) = save_qfix_to_files(&qfix) {
                emit_error(&sender, error).await;
            }
        }
        Task::SaveSelection(target, selection) => {
            if let Err(error) = fs::write(target, selection).await {
                emit_error(&sender, AppError::FileOperationFailed(error)).await;
            }
        }
        Task::TrashPath(entry) => {
            if let Err(error) = cache_and_compress(entry).await {
                emit_error(&sender, error).await;
            }
        }
        Task::YankPath(entry) => {
            if let Err(error) = compress(entry).await {
                emit_error(&sender, error).await;
            }
        }
    };

    Ok(())
}

async fn send_task_started(
    sender: &Sender<Envelope>,
    identifier: &str,
    cancellation: CancellationToken,
) {
    tracing::trace!("task started: {:?}", identifier);

    if identifier == Task::EmitMessages(Vec::new()).to_string() {
        return;
    }

    if let Err(err) = sender
        .send(to_envelope(vec![Message::TaskStarted(
            identifier.to_owned(),
            cancellation,
        )]))
        .await
    {
        tracing::error!("task started send failed: {:?}", err);
    };
}

async fn send_task_finished(sender: &Sender<Envelope>, identifier: &str) {
    tracing::trace!("task ended: {:?}", identifier);

    if identifier == Task::EmitMessages(Vec::new()).to_string() {
        return;
    }

    if let Err(err) = sender
        .send(to_envelope(vec![Message::TaskEnded(identifier.to_owned())]))
        .await
    {
        tracing::error!("task ended send failed: {:?}", err);
    };
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
