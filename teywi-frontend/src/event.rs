use crossterm::event::Event;
use futures::{FutureExt, StreamExt};
use teywi_keymap::{action::Action, conversion, key::Key, ActionResolver};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    Error,
    Key(Key),
    Resize(u16, u16),
    Startup,
}

pub fn listen_crossterm() -> (UnboundedSender<AppEvent>, UnboundedReceiver<AppEvent>) {
    let (sender, receiver) = mpsc::unbounded_channel();
    let internal_sender = sender.clone();

    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();

        internal_sender.send(AppEvent::Startup).unwrap();

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
                            internal_sender.send(AppEvent::Error).unwrap();
                        },
                        None => {},
                    }
                },
            }
        }
    });

    (sender, receiver)
}

pub fn process_appevent(event: AppEvent, action_resolver: &mut ActionResolver) -> Action {
    match event {
        AppEvent::Error => todo!(),
        AppEvent::Key(key) => {
            if let Some(action) = action_resolver.add_and_resolve(key) {
                action
            } else {
                Action::KeySequenceChanged(action_resolver.get_key_string())
            }
        }
        AppEvent::Resize(_, _) => Action::Refresh,
        AppEvent::Startup => Action::Refresh,
    }
}

fn handle_event(event: Event) -> Option<AppEvent> {
    match event {
        Event::Key(key) => {
            if let Some(key) = conversion::to_key(&key) {
                return Some(AppEvent::Key(key));
            }

            None
        }
        Event::Resize(x, y) => Some(AppEvent::Resize(x, y)),
        Event::FocusLost => None,
        Event::FocusGained => None,
        Event::Paste(_s) => None,
        Event::Mouse(_) => None,
    }
}

// let mut client = Client::connect(address).await?;
// client.set("foo", bytes_from_str("bar").unwrap()).await?;

// if let Some(value) = client.get("foo").await? {
//     if let Ok(string) = str::from_utf8(&value) {
//         stdout()
//             .lock()
//             .write_all(format!("\"{}\"", string).as_bytes())?;
//     } else {
//         stdout()
//             .lock()
//             .write_all(format!("{:?}", value).as_bytes())?;
//     }
// } else {
//     stdout().lock().write_all(b"nil")?;

// let _ = stdout().flush();

// fn bytes_from_str(src: &str) -> Result<Bytes, Infallible> {
//     Ok(Bytes::from(src.to_string()))
// }
