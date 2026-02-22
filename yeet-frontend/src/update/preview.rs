use yeet_buffer::model::{ansi::Ansi, BufferLine, TextBuffer};

use crate::{
    action::Action,
    event::Preview,
    model::{App, Buffer, ContentBuffer, PreviewImageBuffer},
    update::app,
};

pub fn update(app: &mut App, content: Preview) -> Vec<Action> {
    match content {
        Preview::Content(path, content) => {
            tracing::trace!("updating preview buffer: {:?}", path);

            let content: Vec<_> = content
                .iter()
                .map(|s| BufferLine {
                    content: Ansi::new(s),
                    ..Default::default()
                })
                .collect();

            let (preview_id, _) = app::get_or_create_directory_buffer(app, &path, &None);
            app.buffers.insert(
                preview_id,
                Buffer::Content(ContentBuffer {
                    path,
                    buffer: TextBuffer {
                        lines: content,
                        ..Default::default()
                    },
                }),
            );
        }
        Preview::Image(path, protocol) => {
            tracing::trace!("updating preview buffer: {:?}", path);

            let (preview_id, _) = app::get_or_create_directory_buffer(app, &path, &None);
            app.buffers.insert(
                preview_id,
                Buffer::Image(PreviewImageBuffer { path, protocol }),
            );
        }
        Preview::None(path) => {
            tracing::trace!("updating preview buffer: {:?}", path);

            let (preview_id, _) = app::get_or_create_directory_buffer(app, &path, &None);
            app.buffers.insert(preview_id, Buffer::PathReference(path));
        }
    }

    Vec::new()
}

pub fn set_buffer_id(app: &mut App, buffer_id: usize) {
    let is_directory = if let Some(Buffer::Directory(it)) = app.buffers.get(&buffer_id) {
        it.path.is_dir()
    } else {
        false
    };

    let preview = app::directory_viewports_mut(app).2;
    preview.buffer_id = buffer_id;
    preview.hide_cursor_line = !is_directory;
}
