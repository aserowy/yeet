use std::path::PathBuf;

use crate::{
    action::Action,
    model::{buffer::BufferLine, Model},
    update::{buffer, history},
};

use super::current;

#[tracing::instrument(skip(model))]
pub fn path(
    model: &mut Model,
    unwatch_old_path: bool,
    watch_new_path: bool,
) -> Option<Vec<Action>> {
    let old_preview_path = model.preview.path.clone();
    let new_preview_path = current::selection(model);

    if old_preview_path == new_preview_path {
        return None;
    }

    model.preview.path = new_preview_path;
    model.preview.buffer.lines.clear();

    tracing::trace!(
        "switching preview path: {:?} -> {:?}",
        old_preview_path,
        model.preview.path
    );

    if let Some(selected) = &model.preview.path {
        let current = &model.current.path;
        if current == selected {
            return None;
        }

        let mut actions = Vec::new();
        if unwatch_old_path {
            if let Some(old) = &old_preview_path {
                actions.push(Action::UnwatchPath(old.clone()));
            }
        }

        if watch_new_path {
            actions.push(Action::WatchPath(selected.clone()));
        }

        Some(actions)
    } else {
        None
    }
}

#[tracing::instrument(skip(model, content))]
pub fn update(model: &mut Model, path: &PathBuf, content: &[String]) -> Option<Vec<Action>> {
    if Some(path) == model.preview.path.as_ref() {
        tracing::trace!("updating preview buffer: {:?}", path);

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
    let target = match &model.preview.path {
        Some(it) => it,
        None => return,
    };

    let buffer = &mut model.preview.buffer;
    let layout = &model.layout.preview;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);
    buffer::reset_view(buffer);

    if !history::set_cursor_index(target, &model.history, buffer) {
        buffer.cursor = None;
    };
}
