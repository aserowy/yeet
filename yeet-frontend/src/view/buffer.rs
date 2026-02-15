use ratatui::{layout::Rect, Frame};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Mode},
    view as buffer_view,
};

use crate::model::{App, Buffer, DirectoryBuffer, Window};

pub fn view(
    mode: &Mode,
    app: &App,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    let (parent_viewport, current_viewport, preview_viewport) = match &app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(parent, current, preview) => (parent, current, preview),
    };

    let parent_buffer =
        app.buffers
            .get(&parent_viewport.buffer_id)
            .and_then(|buffer| match buffer {
                Buffer::Directory(it) => Some(it),
                _ => None,
            });
    let parent_x = parent_viewport.x.saturating_add(horizontal_offset);
    let parent_y = parent_viewport.y.saturating_add(vertical_offset);
    if let Some(buffer) = parent_buffer {
        render_directory_buffer(mode, frame, parent_viewport, buffer, parent_x, parent_y);
    }

    let current_buffer =
        app.buffers
            .get(&current_viewport.buffer_id)
            .and_then(|buffer| match buffer {
                Buffer::Directory(it) => Some(it),
                _ => None,
            });
    let current_x = current_viewport.x.saturating_add(horizontal_offset);
    let current_y = current_viewport.y.saturating_add(vertical_offset);
    if let Some(buffer) = current_buffer {
        render_directory_buffer(mode, frame, current_viewport, buffer, current_x, current_y);
    }

    let preview_buffer = app.buffers.get(&preview_viewport.buffer_id);
    let preview_x = preview_viewport.x.saturating_add(horizontal_offset);
    let preview_y = preview_viewport.y.saturating_add(vertical_offset);
    if let Some(buffer) = preview_buffer {
        match buffer {
            Buffer::Directory(buffer) => {
                render_directory_buffer(
                    mode,
                    frame,
                    preview_viewport,
                    buffer,
                    preview_x,
                    preview_y,
                );
            }
            Buffer::Image(buffer) => {
                let rect = Rect {
                    x: preview_x,
                    y: preview_y,
                    width: preview_viewport.width,
                    height: preview_viewport.height,
                };

                frame.render_widget(Image::new(&buffer.protocol), rect);
            }
            Buffer::Content(buffer) => {
                buffer_view(
                    preview_viewport,
                    mode,
                    &buffer.buffer,
                    frame,
                    preview_x,
                    preview_y,
                );
            }
            Buffer::Empty => {}
        }
    }
}

fn render_directory_buffer(
    mode: &Mode,
    frame: &mut Frame,
    viewport: &ViewPort,
    buffer: &DirectoryBuffer,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    let mut viewport = viewport.clone();
    yeet_buffer::update_viewport_by_cursor(&mut viewport, &buffer.buffer);

    buffer_view(
        &viewport,
        mode,
        &buffer.buffer,
        frame,
        horizontal_offset,
        vertical_offset,
    );
}
