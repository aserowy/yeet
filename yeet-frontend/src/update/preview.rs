use std::path::PathBuf;

use yeet_buffer::{message::BufferMessage, model::BufferLine, update};

use crate::{
    action::Action,
    model::{DirectoryBufferState, Model},
    update::cursor,
};

use super::current;

#[tracing::instrument(skip(model))]
pub fn set_preview_to_selected(model: &mut Model) -> Option<PathBuf> {
    let new = current::get_current_selected_path(model);
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
pub fn update_preview(model: &mut Model, path: &PathBuf, content: &[String]) -> Vec<Action> {
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
        update::update_buffer(
            &model.mode,
            &mut model.files.preview.buffer,
            &BufferMessage::SetContent(content),
        );
        validate_preview_viewport(model);
    }

    Vec::new()
}

pub fn validate_preview_viewport(model: &mut Model) {
    let target = match &model.files.preview.path {
        Some(it) => it,
        None => return,
    };

    let buffer = &mut model.files.preview.buffer;
    let layout = &model.layout.preview;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);
    update::update_buffer(&model.mode, buffer, &BufferMessage::ResetCursor);

    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor_line = true;
    }

    if target.is_dir() {
        cursor::set_cursor_index_with_history(&model.mode, &model.history, buffer, target);
    }
}
