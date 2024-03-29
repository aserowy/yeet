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
    if model.file_buffer.preview.path == new {
        return None;
    }

    let old = model.file_buffer.preview.path.take();
    model.file_buffer.preview.path = new.clone();
    model.file_buffer.preview.buffer.lines.clear();

    tracing::trace!(
        "switching preview path: {:?} -> {:?}",
        old,
        model.file_buffer.preview.path
    );

    new
}

#[tracing::instrument(skip(model, content))]
pub fn update(model: &mut Model, path: &PathBuf, content: &[String]) {
    if Some(path) == model.file_buffer.preview.path.as_ref() {
        tracing::trace!("updating preview buffer: {:?}", path);

        let content = content
            .iter()
            .map(|s| BufferLine {
                content: s.to_string(),
                ..Default::default()
            })
            .collect();

        model.file_buffer.preview.state = DirectoryBufferState::Ready;
        update::update(
            &model.mode,
            &model.search,
            &mut model.file_buffer.preview.buffer,
            &BufferMessage::SetContent(content),
        );
        viewport(model);
    }
}

pub fn viewport(model: &mut Model) {
    let target = match &model.file_buffer.preview.path {
        Some(it) => it,
        None => return,
    };

    let buffer = &mut model.file_buffer.preview.buffer;
    let layout = &model.layout.preview;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);
    update::update(
        &model.mode,
        &model.search,
        buffer,
        &BufferMessage::ResetCursor,
    );

    if !cursor::set_cursor_index_with_history(
        &model.mode,
        &model.history,
        &model.search,
        buffer,
        target,
    ) {
        buffer.cursor = None;
    };
}
