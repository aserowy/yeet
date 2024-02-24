use std::path::PathBuf;

use crate::{
    action::Action,
    model::{buffer::BufferLine, Model},
    update::{buffer, history},
};

use super::current;

pub fn path(
    model: &mut Model,
    unwatch_old_path: bool,
    watch_new_path: bool,
) -> Option<Vec<Action>> {
    if let Some(selected) = current::selection(model) {
        let current = &model.current.path;
        if current == &selected || model.preview.path == selected {
            return None;
        }

        let mut actions = Vec::new();
        if unwatch_old_path {
            actions.push(Action::UnwatchPath(model.preview.path.clone()));
        }

        if watch_new_path {
            actions.extend(vec![
                Action::SleepBeforeRender,
                Action::WatchPath(selected.clone()),
            ]);
        }

        model.preview.path = selected.to_path_buf();

        Some(actions)
    } else {
        None
    }
}

pub fn update(model: &mut Model, path: &PathBuf, content: &[String]) -> Option<Vec<Action>> {
    if path == &model.preview.path {
        let content = content
            .iter()
            .map(|s| BufferLine {
                content: s.to_string(),
                ..Default::default()
            })
            .collect();

        buffer::set_content(&model.mode, &mut model.preview.buffer, content);
        viewport(model);
    }

    None
}

pub fn viewport(model: &mut Model) {
    let target = &model.preview.path;
    let buffer = &mut model.preview.buffer;
    let layout = &model.layout.preview;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);
    buffer::reset_view(buffer);

    if !history::set_cursor_index(target, &model.history, buffer) {
        buffer.cursor = None;
    };
}
