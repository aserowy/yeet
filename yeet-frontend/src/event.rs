use std::{path::Path, sync::Arc};

use futures::{FutureExt, StreamExt};
use notify::{
    event::{ModifyKind, RenameMode},
    RecommendedWatcher, RecursiveMode, Watcher,
};
use tokio::{
    select,
    sync::{
        mpsc::{self, Receiver},
        oneshot, Mutex,
    },
};
use yeet_buffer::model::Mode;
use yeet_keymap::{
    conversion,
    message::{Envelope, KeySequence, Message, MessageSource},
    MessageResolver,
};

use crate::{
    error::AppError,
    init::junkyard::get_junkyard_path,
    task::{Task, TaskManager},
};

pub struct Emitter {
    cancellation: Option<oneshot::Sender<oneshot::Sender<bool>>>,
    tasks: TaskManager,
    pub receiver: Receiver<Envelope>,
    resolver: Arc<Mutex<MessageResolver>>,
    sender: mpsc::Sender<Envelope>,
    watcher: RecommendedWatcher,
}

impl Emitter {
    pub fn start() -> Self {
        let (sender, receiver) = mpsc::channel(1);
        let internal_sender = sender.clone();

        let (watcher_sender, mut notify_receiver) = mpsc::unbounded_channel();
        let watcher = notify::recommended_watcher(move |res| {
            if let Err(error) = watcher_sender.send(res) {
                tracing::error!("sending watched directory changes failed: {:?}", error);
            }
        })
        .expect("Failed to create watcher");

        let resolver = Arc::new(Mutex::new(MessageResolver::default()));

        let (task_sender, mut task_receiver) = mpsc::channel(1);
        let tasks = TaskManager::new(task_sender, resolver.clone());
        tokio::spawn(async move {
            loop {
                let notify_event = notify_receiver.recv().fuse();
                let task_event = task_receiver.recv().fuse();

                tokio::select! {
                    Some(Ok(event)) = notify_event => {
                        if let Some(messages) = handle_notify_event(event) {
                            let _ = internal_sender.send(Envelope {
                                messages,
                                sequence: KeySequence::None,
                                source: MessageSource::Filesystem,
                            }).await;
                        }
                    }
                    event = task_event => {
                        if let Some(envelope) = event {
                            let _ = internal_sender.send(envelope).await;
                        }
                    },
                }
            }
        });

        let (cancellation, cancellation_receiver) = oneshot::channel();
        start_crossterm_listener(cancellation_receiver, resolver.clone(), sender.clone());

        Self {
            cancellation: Some(cancellation),
            sender,
            tasks,
            receiver,
            resolver,
            watcher,
        }
    }

    pub async fn suspend(&mut self) -> Result<bool, oneshot::error::RecvError> {
        if let Some(cancellation) = self.cancellation.take() {
            let (sender, receiver) = oneshot::channel();
            if let Err(error) = cancellation.send(sender) {
                tracing::error!("sending cancellation failed: {:?}", error);
            }

            receiver.await
        } else {
            Ok(false)
        }
    }

    pub fn resume(&mut self) {
        let (cancellation, cancellation_receiver) = oneshot::channel();
        self.cancellation = Some(cancellation);

        start_crossterm_listener(
            cancellation_receiver,
            self.resolver.clone(),
            self.sender.clone(),
        );
    }

    pub async fn shutdown(&mut self) -> Result<(), AppError> {
        self.tasks.finishing().await
    }

    pub async fn set_current_mode(&mut self, mode: Mode) {
        let mut resolver = self.resolver.lock().await;
        resolver.mode = mode;
    }

    pub fn unwatch(&mut self, path: &Path) -> Result<(), AppError> {
        if path != get_junkyard_path()? {
            Ok(self.watcher.unwatch(path)?)
        } else {
            Ok(())
        }
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), AppError> {
        Ok(self.watcher.watch(path, RecursiveMode::NonRecursive)?)
    }

    pub fn run(&mut self, task: Task) {
        self.tasks.run(task);
    }

    pub fn abort(&mut self, task: &Task) {
        self.tasks.abort(task);
    }
}

fn start_crossterm_listener(
    mut cancellation_receiver: oneshot::Receiver<oneshot::Sender<bool>>,
    resolver_mutex: Arc<Mutex<MessageResolver>>,
    sender: mpsc::Sender<Envelope>,
) {
    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();

        loop {
            let crossterm_event = reader.next().fuse();

            select! {
                Ok(sender) = &mut cancellation_receiver => {
                    sender.send(true).expect("Failed to send cancellation signal");
                    break
                }
                Some(Ok(event)) = crossterm_event => {
                    if let Some(envelope) = handle_crossterm_event(&resolver_mutex, event).await {
                        let _ = sender.send(envelope).await;
                    }
                }
            }
        }
    });
}

async fn handle_crossterm_event(
    resolver_mutex: &Arc<Mutex<MessageResolver>>,
    event: crossterm::event::Event,
) -> Option<Envelope> {
    match event {
        crossterm::event::Event::Key(key) => {
            if let Some(key) = conversion::to_key(&key) {
                let mut resolver = resolver_mutex.lock().await;
                let envelope = resolver.add_key(key);
                return Some(envelope);
            }

            None
        }
        crossterm::event::Event::Resize(x, y) => Some(Envelope {
            messages: vec![Message::Resize(x, y)],
            sequence: KeySequence::None,
            source: MessageSource::User,
        }),
        crossterm::event::Event::FocusLost
        | crossterm::event::Event::FocusGained
        | crossterm::event::Event::Paste(_)
        | crossterm::event::Event::Mouse(_) => None,
    }
}

#[tracing::instrument]
fn handle_notify_event(event: notify::Event) -> Option<Vec<Message>> {
    if event.need_rescan() {
        // TODO: Refresh directory
    }

    match event.kind {
        notify::EventKind::Create(_) => Some(
            event
                .paths
                .iter()
                .map(|p| Message::PathsAdded(vec![p.clone()]))
                .collect(),
        ),
        notify::EventKind::Modify(ModifyKind::Name(rename_mode)) => match rename_mode {
            RenameMode::Both => {
                if event.paths.len() == 2 {
                    Some(vec![
                        Message::PathRemoved(event.paths[0].clone()),
                        Message::PathsAdded(vec![event.paths[1].clone()]),
                    ])
                } else {
                    tracing::warn!("event is invalid: {:?}", event);
                    None
                }
            }
            RenameMode::From => {
                if event.paths.len() == 1 {
                    Some(vec![Message::PathRemoved(event.paths[0].clone())])
                } else {
                    tracing::warn!("event is invalid: {:?}", event);
                    None
                }
            }
            RenameMode::To => {
                if event.paths.len() == 1 {
                    Some(vec![Message::PathsAdded(vec![event.paths[0].clone()])])
                } else {
                    tracing::warn!("event is invalid: {:?}", event);
                    None
                }
            }
            RenameMode::Any => {
                if event.paths.len() == 1 {
                    let path = event.paths[0].clone();
                    if path.exists() {
                        Some(vec![Message::PathsAdded(vec![path])])
                    } else {
                        Some(vec![Message::PathRemoved(path)])
                    }
                } else {
                    tracing::warn!("event is invalid: {:?}", event);
                    None
                }
            }
            RenameMode::Other => {
                tracing::trace!("missed handle for notify event: {:?}", event);
                None
            }
        },
        notify::EventKind::Remove(_) => Some(
            event
                .paths
                .iter()
                .map(|p| Message::PathRemoved(p.clone()))
                .collect(),
        ),
        notify::EventKind::Any
        | notify::EventKind::Access(_)
        | notify::EventKind::Modify(_)
        | notify::EventKind::Other => {
            tracing::trace!("missed handle for notify event: {:?}", event);
            None
        }
    }
}
