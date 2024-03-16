use std::path::PathBuf;

use crate::{
    model::{buffer::BufferLine, Model},
    update::{buffer, cursor},
};

use super::current;

#[tracing::instrument(skip(model))]
pub fn selected_path(model: &mut Model) -> Option<PathBuf> {
    let new = current::selection(model);
    if model.preview.path == new {
        return None;
    }

    let old = model.preview.path.clone();
    model.preview.path = new.clone();
    model.preview.buffer.lines.clear();

    tracing::trace!(
        "switching preview path: {:?} -> {:?}",
        old,
        model.preview.path
    );

    new
}

#[tracing::instrument(skip(model, content))]
pub fn update(model: &mut Model, path: &PathBuf, content: &[String]) {
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

    if !cursor::set_cursor_index_with_history(target, &model.history, buffer) {
        buffer.cursor = None;
    };
}
