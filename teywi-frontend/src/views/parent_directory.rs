use std::path::Path;

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
            let path = Path::new(&state.current_directory);
            let parent = path.parent().unwrap().as_os_str();

            state.parent_directory_state.paths = std::fs::read_dir(parent)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
        }
        _ => {}
    }

    None
}

pub fn view(state: &mut AppState, frame: &mut Frame, rect: Rect) {
    let paths: Vec<ListItem> = state
        .parent_directory_state
        .paths
        .iter()
        .map(|path| ListItem::new(path.file_name().unwrap().to_str().unwrap()))
        .collect();

    frame.render_stateful_widget(
        List::new(paths),
        rect,
        &mut state.parent_directory_state.state,
    )
}
