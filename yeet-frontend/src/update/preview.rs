use std::path::PathBuf;

use yeet_buffer::{
    message::BufferMessage,
    model::{ansi::Ansi, BufferLine},
    update::update_buffer,
};
use yeet_keymap::message::Preview;

use crate::{
    action::Action,
    model::{DirectoryBufferState, Model},
    update::selection::get_current_selected_path,
};

use super::{cursor::set_cursor_index_with_history, set_viewport_dimensions};

#[tracing::instrument(skip(model))]
pub fn set_preview_to_selected(model: &mut Model) -> Option<PathBuf> {
    let new = get_current_selected_path(model);
    if model.files.preview.path == new {
        return None;
    }

    let old = model.files.preview.path.take();
    model.files.preview.path.clone_from(&new);
    model.files.preview.buffer.lines.clear();

    tracing::trace!(
        "switching preview path: {:?} -> {:?}",
        old,
        model.files.preview.path
    );

    new
}

#[tracing::instrument(skip(model, content))]
pub fn update_preview(model: &mut Model, path: &PathBuf, content: &Preview) -> Vec<Action> {
    if Some(path) == model.files.preview.path.as_ref() {
        tracing::trace!("updating preview buffer: {:?}", path);

        match content {
            Preview::Content(content) => {
                let content = content
                    .iter()
                    .map(|s| BufferLine {
                        content: Ansi::new(s),
                        ..Default::default()
                    })
                    .collect();

                // FIX: why here? this should get handled with enumeration or file preview
                model.files.preview.state = DirectoryBufferState::Ready;
                update_buffer(
                    &model.mode,
                    &mut model.files.preview.buffer,
                    &BufferMessage::SetContent(content),
                );
                validate_preview_viewport(model);
            }
            Preview::None => {}
        };
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

    set_viewport_dimensions(&mut buffer.view_port, layout);
    update_buffer(&model.mode, buffer, &BufferMessage::ResetCursor);

    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor_line = true;
    }

    if target.is_dir() {
        set_cursor_index_with_history(&model.mode, &model.history, buffer, target);
    }
}
