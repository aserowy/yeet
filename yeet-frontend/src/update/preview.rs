use std::path::PathBuf;

use yeet_buffer::{message::BufferMessage, model::BufferLine, update};

use crate::{
    model::{DirectoryBufferState, Model},
    update::cursor,
};

use super::current;

#[tracing::instrument(skip(model))]
pub fn selected_path(model: &mut Model) -> Option<PathBuf> {
    let new = current::selection(model);
    if model.files.preview.path == new {
        return None;
    }

    let old = model.files.preview.path.take();
    model.files.preview.path = new.clone();
    model.files.preview.buffer.lines.clear();

    tracing::trace!(
        "switching preview path: {:?} -> {:?}",
        old,
        model.files.preview.path
    );

    new
}

#[tracing::instrument(skip(model, content))]
pub fn update(model: &mut Model, path: &PathBuf, content: &[String]) {
    if Some(path) == model.files.preview.path.as_ref() {
        tracing::trace!("updating preview buffer: {:?}", path);

        let content = content
            .iter()
            .map(|s| BufferLine {
                content: s.to_string(),
                ..Default::default()
            })
            .collect();

        model.files.preview.state = DirectoryBufferState::Ready;
        update::update(
            &model.mode,
            &mut model.files.preview.buffer,
            &BufferMessage::SetContent(content),
        );
        viewport(model);
    }
}

pub fn viewport(model: &mut Model) {
    let target = match &model.files.preview.path {
        Some(it) => it,
        None => return,
    };

    let buffer = &mut model.files.preview.buffer;
    let layout = &model.layout.preview;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);
    update::update(&model.mode, buffer, &BufferMessage::ResetCursor);

    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor_line = true;
    }

    if target.is_dir() {
        cursor::set_cursor_index_with_history(&model.mode, &model.history, buffer, target);
    }
}
