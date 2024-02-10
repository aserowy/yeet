use std::{path::PathBuf, time::Duration};

use futures::{FutureExt, StreamExt};
use notify::PollWatcher;
use tokio::sync::mpsc::{self, UnboundedReceiver};
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
pub fn listen() -> (PollWatcher, UnboundedReceiver<RenderAction>) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let internal_sender = sender.clone();

    let (watcher_sender, mut watcher_receiver) = mpsc::unbounded_channel();
    let watcher = PollWatcher::new(
        move |res| {
            watcher_sender.send(res).unwrap();
        },
        notify::Config::default().with_poll_interval(Duration::from_millis(500)),
    )
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
                            if let Some(message) = handle_event(event) {
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
                        Some(Ok(_event)) => {
                            // TODO: handle notify event and replace single buffer lines
                            internal_sender.send(RenderAction::Refresh).unwrap();
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

    (watcher, receiver)
}

fn handle_event(event: crossterm::event::Event) -> Option<RenderAction> {
    match event {
        crossterm::event::Event::Key(key) => {
            if let Some(key) = conversion::to_key(&key) {
                return Some(RenderAction::Key(key));
            }

            None
        }
        crossterm::event::Event::Resize(x, y) => Some(RenderAction::Resize(x, y)),
        crossterm::event::Event::FocusLost => None,
        crossterm::event::Event::FocusGained => None,
        crossterm::event::Event::Paste(_) => None,
        crossterm::event::Event::Mouse(_) => None,
    }
}

pub fn convert_to_messages(
    event: RenderAction,
    message_resolver: &mut MessageResolver,
) -> Vec<Message> {
    match event {
        // TODO: log error?
        RenderAction::Error => vec![Message::Refresh],
        RenderAction::Key(key) => message_resolver.add_and_resolve(key),
        RenderAction::Refresh => vec![Message::Refresh],
        RenderAction::Resize(_, _) => vec![Message::Refresh],
        RenderAction::Startup => vec![Message::Startup],
    }
}
