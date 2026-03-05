use std::collections::HashMap;

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
    render_window(
        mode,
        &app.window,
        &app.contents.buffers,
        frame,
        horizontal_offset,
        vertical_offset,
    );
}

fn render_window(
    mode: &Mode,
    window: &Window,
    buffers: &HashMap<usize, Buffer>,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    match window {
        Window::Horizontal { first, second, .. } => {
            render_window(
                mode,
                first,
                buffers,
                frame,
                horizontal_offset,
                vertical_offset,
            );
            render_window(
                mode,
                second,
                buffers,
                frame,
                horizontal_offset,
                vertical_offset,
            );
        }
        Window::Directory(parent, current, preview) => {
            render_buffer_slot(
                mode,
                frame,
                parent,
                buffers.get(&parent.buffer_id),
                horizontal_offset,
                vertical_offset,
            );
            render_buffer_slot(
                mode,
                frame,
                current,
                buffers.get(&current.buffer_id),
                horizontal_offset,
                vertical_offset,
            );
            render_buffer_slot(
                mode,
                frame,
                preview,
                buffers.get(&preview.buffer_id),
                horizontal_offset,
                vertical_offset,
            );
        }
        Window::Tasks(vp) => {
            render_buffer_slot(
                mode,
                frame,
                vp,
                buffers.get(&vp.buffer_id),
                horizontal_offset,
                vertical_offset,
            );
        }
    }
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
        Some(Buffer::Tasks(_)) | Some(Buffer::PathReference(_)) | Some(Buffer::Empty) | None => {
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
