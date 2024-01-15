use ratatui::{
    prelude::Rect,
    widgets::{List, ListItem},
    Frame,
};

use crate::state::{AppState, Message};

pub fn update(state: &mut AppState, message: Message) -> Option<Message> {
    match message {
        Message::Startup => {
            // TODO: handle errors and remove unwrap
            state.current_directory_state.paths = std::fs::read_dir(&state.current_directory)
                .unwrap()
                .map(|entry| entry.unwrap().path().display().to_string())
                .collect();
        }
        _ => {}
    }

    None
}

pub fn view(state: &mut AppState, frame: &mut Frame, rect: Rect) {
    let paths: Vec<ListItem> = state
        .current_directory_state
        .paths
        .iter()
        .map(|path| ListItem::new(path.as_str()))
        .collect();

    frame.render_stateful_widget(
        List::new(paths),
        rect,
        &mut state.current_directory_state.state,
    )
}
