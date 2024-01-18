use crossterm::event::{Event, KeyCode, KeyEventKind, MouseEvent};
use futures::{FutureExt, StreamExt};
use teywi_keymap::{action::Action, conversion};
use tokio::sync::mpsc::{self, UnboundedReceiver};

#[derive(Clone, Debug)]
pub enum AppEvent {
    Action(Action),
    Error,
    Mouse(MouseEvent),
    Resize(u16, u16),
    Startup,
    Quit,
}

pub fn start() -> UnboundedReceiver<AppEvent> {
    let (sender, receiver) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();

        sender.send(AppEvent::Startup).unwrap();

        loop {
            let crossterm_event = reader.next().fuse();
            // TODO: let notify dict changed
            // TODO: let backend state changed events (see comments bottom)

            tokio::select! {
                event = crossterm_event => {
                    match event {
                        Some(Ok(event)) => {
                            if let Some(message) = handle_event(event) {
                                sender.send(message).unwrap();
                            }
                        },
                        Some(Err(_)) => {
                            sender.send(AppEvent::Error).unwrap();
                        },
                        None => {},
                    }
                },
            }
        }
    });

    receiver
}

fn handle_event(event: Event) -> Option<AppEvent> {
    match event {
        Event::Key(key) => {
            let _keypress = conversion::to_key(key.clone());
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Char('q') {
                    return Some(AppEvent::Quit);
                } else {
                    return Some(AppEvent::Action(Action::Refresh));
                }
            }

            None
        }
        Event::Mouse(mouse) => Some(AppEvent::Mouse(mouse)),
        Event::Resize(x, y) => Some(AppEvent::Resize(x, y)),
        Event::FocusLost => None,
        Event::FocusGained => None,
        Event::Paste(_s) => None,
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
