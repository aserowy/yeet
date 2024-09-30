use std::path::Path;

use yeet_buffer::{
    message::BufferMessage,
    model::{ansi::Ansi, BufferLine},
    update::update_buffer,
};

use crate::{
    action::Action,
    event::Preview,
    model::{DirectoryBuffer, DirectoryBufferState, Model, PreviewContent},
    update::viewport,
};

use super::cursor::set_cursor_index_with_history;

pub fn set_buffer(model: &mut Model, path: &Path, content: Vec<BufferLine>) {
    let mut dir = DirectoryBuffer::default();
    dir.path = path.to_path_buf();
    dir.state = DirectoryBufferState::Ready;

    update_buffer(
        &model.mode,
        &mut dir.buffer,
        &BufferMessage::SetContent(content.to_vec()),
    );

    model.files.preview = PreviewContent::Buffer(dir);

    update_cursor_and_viewport(model);
}

#[tracing::instrument(skip(model, content))]
pub fn update_preview(model: &mut Model, content: Preview) -> Vec<Action> {
    match content {
        Preview::Content(path, content) => {
            tracing::trace!("updating preview buffer: {:?}", path);

            let content = content
                .iter()
                .map(|s| BufferLine {
                    content: Ansi::new(s),
                    ..Default::default()
                })
                .collect();

            set_buffer(model, &path, content);
        }
        Preview::Image(path, protocol) => {
            model.files.preview = PreviewContent::Image(path, protocol)
        }
        Preview::None(_) => model.files.preview = PreviewContent::None,
    };
    Vec::new()
}

pub fn update_cursor_and_viewport(model: &mut Model) {
    let dir = match &mut model.files.preview {
        PreviewContent::Buffer(it) => it,
        _ => return,
    };

    let buffer = &mut dir.buffer;
    let layout = &model.layout.preview;

    tracing::warn!("{:?}", &model.layout.preview);
    viewport::set_viewport_dimensions(&mut buffer.view_port, layout);
    update_buffer(&model.mode, buffer, &BufferMessage::ResetCursor);
    tracing::warn!("{:?}", &buffer);
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor_line = true;
    }

    if dir.path.is_dir() {
        set_cursor_index_with_history(&model.mode, &model.history, buffer, dir.path.as_path());
    }
}
