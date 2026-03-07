use std::collections::HashMap;

use ratatui::{layout::Rect, Frame};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Mode},
    view as buffer_view,
};

use crate::model::{App, Buffer, DirectoryBuffer, SplitFocus, Window};

use super::statusline;

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
        true,
        false,
    );
}

fn render_window(
    mode: &Mode,
    window: &Window,
    buffers: &HashMap<usize, Buffer>,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
    is_focused: bool,
    draw_borders: bool,
) {
    match window {
        Window::Horizontal {
            first,
            second,
            focus,
        } => {
            render_window(
                mode,
                first,
                buffers,
                frame,
                horizontal_offset,
                vertical_offset,
                is_focused && focus == &SplitFocus::First,
                false,
            );
            render_window(
                mode,
                second,
                buffers,
                frame,
                horizontal_offset,
                vertical_offset,
                is_focused && focus == &SplitFocus::Second,
                false,
            );
        }
        Window::Vertical {
            first,
            second,
            focus,
        } => {
            render_window(
                mode,
                first,
                buffers,
                frame,
                horizontal_offset,
                vertical_offset,
                is_focused && focus == &SplitFocus::First,
                true,
            );
            render_window(
                mode,
                second,
                buffers,
                frame,
                horizontal_offset,
                vertical_offset,
                is_focused && focus == &SplitFocus::Second,
                false,
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
                is_focused,
                None,
            );
            render_buffer_slot(
                mode,
                frame,
                current,
                buffers.get(&current.buffer_id),
                horizontal_offset,
                vertical_offset,
                is_focused,
                None,
            );
            render_buffer_slot(
                mode,
                frame,
                preview,
                buffers.get(&preview.buffer_id),
                horizontal_offset,
                vertical_offset,
                is_focused,
                Some(draw_borders),
            );

            if let Some(buffer) = buffers.get(&current.buffer_id) {
                let total_width = (preview.x + preview.width).saturating_sub(parent.x);
                let statusline_rect = Rect {
                    x: parent.x.saturating_add(horizontal_offset),
                    y: current
                        .y
                        .saturating_add(current.height)
                        .saturating_add(vertical_offset),
                    width: total_width,
                    height: 1,
                };
                statusline::view(buffer, current, frame, statusline_rect, is_focused);
            }
        }
        Window::Tasks(vp) => {
            render_buffer_slot(
                mode,
                frame,
                vp,
                buffers.get(&vp.buffer_id),
                horizontal_offset,
                vertical_offset,
                is_focused,
                Some(draw_borders),
            );

            if let Some(buffer) = buffers.get(&vp.buffer_id) {
                let statusline_rect = Rect {
                    x: vp.x.saturating_add(horizontal_offset),
                    y: vp
                        .y
                        .saturating_add(vp.height)
                        .saturating_add(vertical_offset),
                    width: vp.width,
                    height: 1,
                };
                statusline::view(buffer, vp, frame, statusline_rect, is_focused);
            }
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
    is_focused: bool,
    draw_border: Option<bool>,
) {
    let x = viewport.x.saturating_add(horizontal_offset);
    let y = viewport.y.saturating_add(vertical_offset);

    let mut effective_vp = if is_focused {
        viewport.clone()
    } else {
        ViewPort {
            hide_cursor: true,
            hide_cursor_line: true,
            ..viewport.clone()
        }
    };

    if let Some(true) = draw_border {
        effective_vp.show_border = true;
        effective_vp.width = effective_vp.width.saturating_sub(1);
    }

    match buffer {
        Some(Buffer::Content(buffer)) => {
            buffer_view(&effective_vp, mode, &buffer.buffer, frame, x, y);
        }
        Some(Buffer::Directory(buffer)) => {
            render_directory_buffer(mode, frame, &effective_vp, buffer, x, y);
        }
        Some(Buffer::Tasks(tasks_buf)) => {
            buffer_view(&effective_vp, mode, &tasks_buf.buffer, frame, x, y);
        }
        Some(Buffer::PathReference(_)) | Some(Buffer::Empty) | None => {
            let mut vp = effective_vp.clone();
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
