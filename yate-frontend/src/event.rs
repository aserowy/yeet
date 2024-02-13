use std::{env, path::PathBuf};

use futures::{FutureExt, StreamExt};
use notify::{
    event::{ModifyKind, RenameMode},
    INotifyWatcher,
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use yate_keymap::{
    conversion,
    key::Key,
    message::{Message, Mode},
    MessageResolver,
};

use crate::task::Task;

#[derive(Clone, Debug, PartialEq)]
pub enum RenderAction {
    Error,
    Key(Key),
    Resize(u16, u16),
    Refresh,
    Startup,
    PathsAdded(Vec<PathBuf>),
    PathRemoved(PathBuf),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PostRenderAction {
    ModeChanged(Mode),
    OptimizeHistory,
    Quit,
    Task(Task),
    UnwatchPath(PathBuf),
    WatchPath(PathBuf),
}

// TODO: replace unwraps with shutdown struct (server) and graceful exit 1
pub fn listen() -> (
    INotifyWatcher,
    UnboundedSender<RenderAction>,
    UnboundedReceiver<RenderAction>,
) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let internal_sender = sender.clone();

    let (watcher_sender, mut watcher_receiver) = mpsc::unbounded_channel();
    let watcher = notify::recommended_watcher(move |res| {
        watcher_sender.send(res).unwrap();
    })
    .unwrap();

    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();

        internal_sender.send(RenderAction::Startup).unwrap();

        loop {
            let crossterm_event = reader.next().fuse();
            let notify_event = watcher_receiver.recv().fuse();

            tokio::select! {
                event = crossterm_event => {
                    match event {
                        Some(Ok(event)) => {
                            if let Some(message) = handle_crossterm_event(event) {
                                internal_sender.send(message).unwrap();
                            }
                        },
                        Some(Err(_)) => {
                            internal_sender.send(RenderAction::Error).unwrap();
                        },
                        None => {},
                    }
                },
                event = notify_event => {
                    match event {
                        Some(Ok(event)) => {
                            if let Some(messages) = handle_notify_event(event) {
                                for message in messages {
                                    internal_sender.send(message).unwrap();
                                }
                            }
                        },
                        Some(Err(_)) => {
                            internal_sender.send(RenderAction::Error).unwrap();
                        },
                        None => {},
                    }
                },

            }
        }
    });

    (watcher, sender, receiver)
}

fn handle_crossterm_event(event: crossterm::event::Event) -> Option<RenderAction> {
    match event {
        crossterm::event::Event::Key(key) => {
            if let Some(key) = conversion::to_key(&key) {
                return Some(RenderAction::Key(key));
            }

            None
        }
        crossterm::event::Event::Resize(x, y) => Some(RenderAction::Resize(x, y)),
        crossterm::event::Event::FocusLost
        | crossterm::event::Event::FocusGained
        | crossterm::event::Event::Paste(_)
        | crossterm::event::Event::Mouse(_) => None,
    }
}

fn handle_notify_event(event: notify::Event) -> Option<Vec<RenderAction>> {
    if event.need_rescan() {
        return Some(vec![RenderAction::Refresh]);
    }

    match event.kind {
        notify::EventKind::Create(_) => Some(
            event
                .paths
                .iter()
                .map(|p| RenderAction::PathsAdded(vec![p.clone()]))
                .collect(),
        ),
        // TODO: handle rename events with rename mode to/from (needs buffering)
        notify::EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            if event.paths.len() == 2 {
                Some(vec![
                    RenderAction::PathRemoved(event.paths[0].clone()),
                    RenderAction::PathsAdded(vec![event.paths[1].clone()]),
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
                .map(|p| RenderAction::PathRemoved(p.clone()))
                .collect(),
        ),
        notify::EventKind::Any
        | notify::EventKind::Access(_)
        | notify::EventKind::Modify(_)
        | notify::EventKind::Other => None,
    }
}

pub fn convert_to_messages(
    event: RenderAction,
    message_resolver: &mut MessageResolver,
) -> Vec<Message> {
    match event {
        // TODO: log error?
        RenderAction::Error => vec![],
        RenderAction::Key(key) => message_resolver.add_and_resolve(key),
        RenderAction::PathsAdded(paths) => vec![Message::PathsAdded(paths)],
        RenderAction::PathRemoved(path) => vec![Message::PathRemoved(path)],
        RenderAction::Refresh => vec![],
        RenderAction::Resize(_, _) => vec![],
        RenderAction::Startup => vec![Message::SelectPath(get_current_path())],
    }
}

fn get_current_path() -> PathBuf {
    // TODO: configurable with clap
    if let Ok(path) = env::current_dir() {
        path
    } else if let Some(val) = dirs::home_dir() {
        val
    } else {
        // TODO: log error
        PathBuf::new()
    }
}
