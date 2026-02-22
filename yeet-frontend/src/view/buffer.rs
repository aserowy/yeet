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

    let parent_buffer = app.buffers.get(&parent_viewport.buffer_id);
    render_buffer_slot(
        mode,
        frame,
        parent_viewport,
        parent_buffer,
        horizontal_offset,
        vertical_offset,
    );

    let current_buffer = app.buffers.get(&current_viewport.buffer_id);
    render_buffer_slot(
        mode,
        frame,
        current_viewport,
        current_buffer,
        horizontal_offset,
        vertical_offset,
    );

    let preview_buffer = app.buffers.get(&preview_viewport.buffer_id);
    render_buffer_slot(
        mode,
        frame,
        preview_viewport,
        preview_buffer,
        horizontal_offset,
        vertical_offset,
    );
}

fn render_buffer_slot(
    mode: &Mode,
    frame: &mut Frame,
    viewport: &ViewPort,
    buffer: Option<&Buffer>,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    let x = viewport.x.saturating_add(horizontal_offset);
    let y = viewport.y.saturating_add(vertical_offset);

    match buffer {
        Some(Buffer::Content(buffer)) => {
            buffer_view(viewport, mode, &buffer.buffer, frame, x, y);
        }
        Some(Buffer::Directory(buffer)) => {
            render_directory_buffer(mode, frame, viewport, buffer, x, y);
        }
        Some(Buffer::PathReference(_)) | Some(Buffer::Empty) | None => {
            let mut vp = viewport.clone();
            vp.hide_cursor = true;
            vp.hide_cursor_line = true;

            render_directory_buffer(mode, frame, &vp, &Default::default(), x, y);
        }
        Some(Buffer::Image(buffer)) => {
            let rect = Rect {
                x,
                y,
                width: viewport.width,
                height: viewport.height,
            };

            frame.render_widget(Image::new(&buffer.protocol), rect);
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
