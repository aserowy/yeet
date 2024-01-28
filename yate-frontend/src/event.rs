use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use yate_keymap::{
    conversion,
    key::Key,
    message::{Message, Mode},
    MessageResolver,
};

#[derive(Clone, Debug, PartialEq)]
pub enum RenderAction {
    Error,
    Key(Key),
    Resize(u16, u16),
    Startup,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PostRenderAction {
    ModeChanged(Mode),
    Quit,
}

pub fn listen() -> (UnboundedSender<RenderAction>, UnboundedReceiver<RenderAction>) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let internal_sender = sender.clone();

    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();

        internal_sender.send(RenderAction::Startup).unwrap();

        loop {
            let crossterm_event = reader.next().fuse();
            // TODO: let notify dict changed
            // TODO: let backend state changed events (see comments bottom)

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
            }
        }
    });

    (sender, receiver)
}

pub fn convert(event: RenderAction, message_resolver: &mut MessageResolver) -> Vec<Message> {
    match event {
        RenderAction::Error => todo!(),
        RenderAction::Key(key) => message_resolver.add_and_resolve(key),
        RenderAction::Resize(_, _) => vec![Message::Refresh],
        RenderAction::Startup => vec![Message::Refresh],
    }
}

fn handle_event(event: Event) -> Option<RenderAction> {
    match event {
        Event::Key(key) => {
            if let Some(key) = conversion::to_key(&key) {
                return Some(RenderAction::Key(key));
            }

            None
        }
        Event::Resize(x, y) => Some(RenderAction::Resize(x, y)),
        Event::FocusLost => None,
        Event::FocusGained => None,
        Event::Paste(_s) => None,
        Event::Mouse(_) => None,
    }
}
