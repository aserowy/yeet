use ratatui::{
    prelude::Rect,
    widgets::{List, ListItem},
    Frame,
};

use crate::model::Model;

pub fn view(model: &mut Model, frame: &mut Frame, rect: Rect) {
    let paths: Vec<ListItem> = model
        .current_directory
        .paths
        .iter()
        .map(|path| ListItem::new(path.file_name().unwrap().to_str().unwrap()))
        .collect();

    frame.render_stateful_widget(
        List::new(paths),
        rect,
        &mut model.current_directory.state,
    )
}
