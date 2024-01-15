use ratatui::{
    prelude::Rect,
    widgets::{List, ListItem},
    Frame,
};

use crate::state::AppState;

pub fn update(state: &mut AppState, message: Event) -> Option<Event> {
    match message {
        Message::Startup => {
            // TODO: handle errors and remove unwrap
            state.current_directory_state.paths = std::fs::read_dir(&state.current_directory)
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
        .current_directory_state
        .paths
        .iter()
        .map(|path| ListItem::new(path.file_name().unwrap().to_str().unwrap()))
        .collect();

    frame.render_stateful_widget(
        List::new(paths),
        rect,
        &mut state.current_directory_state.state,
    )
}
