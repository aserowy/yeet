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
              event = crossterm_event => { match event {
                  Some(Ok(event)) => {
                    match event {
                      Event::Key(key) => {
                        // TODO: handle in keymap crate
                        if key.kind == KeyEventKind::Press {
                            if key.code == KeyCode::Char('q') {
                                sender.send(Message::Quit).unwrap();
                            } else {
                              sender.send(Message::Key).unwrap();
                            }
                        }
                      },
                      Event::Mouse(mouse) => {
                        sender.send(Message::Mouse(mouse)).unwrap();
                      },
                      Event::Resize(x, y) => {
                        sender.send(Message::Resize(x, y)).unwrap();
                      },
                      Event::FocusLost => {
                      },
                      Event::FocusGained => {
                      },
                      Event::Paste(_s) => {
                      },
                    }
                  }
                  Some(Err(_)) => {
                    sender.send(Message::Error).unwrap();
                  }
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
