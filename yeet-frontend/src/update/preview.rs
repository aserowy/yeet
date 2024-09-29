use std::path::Path;

use yeet_buffer::{
    message::BufferMessage,
    model::{ansi::Ansi, BufferLine, Mode},
    update::update_buffer,
};

use crate::{
    action::Action,
    event::Preview,
    model::{DirectoryBuffer, DirectoryBufferState, Model, PreviewContent},
};

use super::{cursor::set_cursor_index_with_history, set_viewport_dimensions};

pub fn create_buffer(mode: &Mode, path: &Path, content: Vec<BufferLine>) -> PreviewContent {
    let mut dir = DirectoryBuffer::default();
    dir.path = path.to_path_buf();
    dir.state = DirectoryBufferState::Ready;

    update_buffer(
        mode,
        &mut dir.buffer,
        &BufferMessage::SetContent(content.to_vec()),
    );
    PreviewContent::Buffer(dir)
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

            model.files.preview = create_buffer(&model.mode, &path, content);
            validate_preview_viewport(model);
        }
        Preview::Image(path, protocol) => {
            model.files.preview = PreviewContent::Image(path, protocol)
        }
        Preview::None(_) => model.files.preview = PreviewContent::None,
    };
    Vec::new()
}

pub fn validate_preview_viewport(model: &mut Model) {
    let dir = match &mut model.files.preview {
        PreviewContent::Buffer(it) => it,
        _ => return,
    };

    let buffer = &mut dir.buffer;
    let layout = &model.layout.preview;

    set_viewport_dimensions(&mut buffer.view_port, layout);
    update_buffer(&model.mode, buffer, &BufferMessage::ResetCursor);

    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor_line = true;
    }

    if dir.path.is_dir() {
        set_cursor_index_with_history(&model.mode, &model.history, buffer, dir.path.as_path());
    }
}
