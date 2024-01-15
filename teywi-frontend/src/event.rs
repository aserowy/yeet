use std::error::Error;

use crossterm::event::{Event, KeyCode, KeyEventKind, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::{self, UnboundedReceiver};

#[derive(Clone, Debug)]
pub enum Message {
    Error,
    Key,
    Mouse(MouseEvent),
    Render,
    Resize(u16, u16),
    Startup,
    Tick,
    Quit,
}

pub fn start() -> UnboundedReceiver<Message> {
    let tick_delay = std::time::Duration::from_secs_f64(1.0 / 4.0);
    let render_delay = std::time::Duration::from_secs_f64(1.0 / 60.0);

    let (sender, receiver) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut reader = crossterm::event::EventStream::new();
        let mut tick_interval = tokio::time::interval(tick_delay);
        let mut render_interval = tokio::time::interval(render_delay);

        sender.send(Message::Startup).unwrap();

        loop {
            let tick_delay = tick_interval.tick();
            let render_delay = render_interval.tick();
            let crossterm_event = reader.next().fuse();

            tokio::select! {
              event = crossterm_event => {
                match event {
                    Some(Ok(event)) => {
                        if let Some(message) = handle_event(event) {
                            sender.send(message).unwrap();
                        }
                    },
                    Some(Err(_)) => {
                        sender.send(Message::Error).unwrap();
                    },
                    None => {},
                }
              },
              _ = tick_delay => {
                  sender.send(Message::Tick).unwrap();
              },
              _ = render_delay => {
                  sender.send(Message::Render).unwrap();
              },
            }
        }
    });

    receiver
}

fn handle_event(event: Event) -> Option<Message> {
    match event {
        // TODO: handle in keymap crate and add action to Key message
        Event::Key(key) => {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Char('q') {
                    return Some(Message::Quit);
                } else {
                    return Some(Message::Key);
                }
            }

            None
        }
        Event::Mouse(mouse) => Some(Message::Mouse(mouse)),
        Event::Resize(x, y) => Some(Message::Resize(x, y)),
        Event::FocusLost => None,
        Event::FocusGained => None,
        Event::Paste(_s) => None,
    }
}

// TODO: add server streams to event system
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
