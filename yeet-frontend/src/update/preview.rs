use crate::{
    model::{App, Buffer},
    update::app,
};

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
