use std::{path::PathBuf, sync::Arc};

use futures::{FutureExt, StreamExt};
use notify::{
    event::{ModifyKind, RenameMode},
    RecommendedWatcher,
};
use tokio::sync::{
    mpsc::{self, Receiver},
    Mutex,
};
use y1337_keymap::{
    conversion,
    message::{Message, Mode},
    MessageResolver,
};

use crate::task::{Task, TaskManager};

#[derive(Clone, Debug, PartialEq)]
pub enum RenderAction {
    Pre(PreRenderAction),
    Post(PostRenderAction),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreRenderAction {
    Resize(u16, u16),
    SleepBeforeRender,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PostRenderAction {
    ModeChanged(Mode),
    Open(PathBuf),
    Quit(Option<String>),
    Task(Task),
    UnwatchPath(PathBuf),
    WatchPath(PathBuf),
}

pub fn listen(
    initial_path: PathBuf,
) -> (
    Arc<Mutex<MessageResolver>>,
    RecommendedWatcher,
    TaskManager,
    Receiver<Vec<Message>>,
) {
    let resolver_mutex = Arc::new(Mutex::new(MessageResolver::default()));
    let inner_resolver_mutex = resolver_mutex.clone();

    let (sender, receiver) = mpsc::channel(1);
    let internal_sender = sender.clone();

    let (watcher_sender, mut notify_receiver) = mpsc::unbounded_channel();
    let watcher = notify::recommended_watcher(move |res| {
        if let Err(_err) = watcher_sender.send(res) {
            // TODO: log error
        }
    })
    .expect("Failed to create watcher");

    let (task_sender, mut task_receiver) = mpsc::channel(1);
    let tasks = TaskManager::new(task_sender);

    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();

        internal_sender
            .send(vec![Message::SelectPath(initial_path)])
            .await
            .expect("Failed to send message");

        loop {
            let crossterm_event = reader.next().fuse();
            let notify_event = notify_receiver.recv().fuse();
            let task_event = task_receiver.recv().fuse();

            tokio::select! {
                Some(Ok(event)) = crossterm_event => {
                    if let Some(message) = handle_crossterm_event(&inner_resolver_mutex, event).await {
                        let _ = internal_sender.send(message).await;
                    }
                },
                Some(Ok(event)) = notify_event => {
                    if let Some(messages) = handle_notify_event(event) {
                        let _ = internal_sender.send(messages).await;
                    }
                }
                event = task_event => {
                    if let Some(event) = event{
                        let _ = internal_sender.send(event).await;
                    }
                },
            }
        }
    });

    (resolver_mutex.clone(), watcher, tasks, receiver)
}

async fn handle_crossterm_event(
    resolver_mutex: &Arc<Mutex<MessageResolver>>,
    event: crossterm::event::Event,
) -> Option<Vec<Message>> {
    match event {
        crossterm::event::Event::Key(key) => {
            if let Some(key) = conversion::to_key(&key) {
                let mut resolver = resolver_mutex.lock().await;
                let messages = resolver.add_and_resolve(key);
                return Some(messages);
            }

            None
        }
        crossterm::event::Event::Resize(x, y) => Some(vec![Message::Resize(x, y)]),
        crossterm::event::Event::FocusLost
        | crossterm::event::Event::FocusGained
        | crossterm::event::Event::Paste(_)
        | crossterm::event::Event::Mouse(_) => None,
    }
}

fn handle_notify_event(event: notify::Event) -> Option<Vec<Message>> {
    if event.need_rescan() {
        // TODO: Refresh directory states
    }

    match event.kind {
        notify::EventKind::Create(_) => Some(
            event
                .paths
                .iter()
                .map(|p| Message::PathsAdded(vec![p.clone()]))
                .collect(),
        ),
        notify::EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            if event.paths.len() == 2 {
                Some(vec![
                    Message::PathRemoved(event.paths[0].clone()),
                    Message::PathsAdded(vec![event.paths[1].clone()]),
                ])
            } else {
                // TODO: log invalid event
                None
            }
        }
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
        | notify::EventKind::Other => None,
    }
}
