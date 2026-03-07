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
    let context = RenderContext {
        draw_borders: None,
        is_focused: true,
        horizontal_offset,
        vertical_offset,
    };

    render_window(mode, &app.window, &app.contents.buffers, frame, context);
}

#[derive(Clone)]
struct RenderContext {
    draw_borders: Option<bool>,
    is_focused: bool,
    horizontal_offset: u16,
    vertical_offset: u16,
}

fn render_window(
    mode: &Mode,
    window: &Window,
    buffers: &HashMap<usize, Buffer>,
    frame: &mut Frame,
    context: RenderContext,
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
                RenderContext {
                    is_focused: context.is_focused && focus == &SplitFocus::First,
                    ..context.clone()
                },
            );
            render_window(
                mode,
                second,
                buffers,
                frame,
                RenderContext {
                    is_focused: context.is_focused && focus == &SplitFocus::Second,
                    ..context.clone()
                },
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
                RenderContext {
                    is_focused: context.is_focused && focus == &SplitFocus::First,
                    draw_borders: Some(true),
                    ..context.clone()
                },
            );
            render_window(
                mode,
                second,
                buffers,
                frame,
                RenderContext {
                    is_focused: context.is_focused && focus == &SplitFocus::Second,
                    ..context.clone()
                },
            );
        }
        Window::Directory(parent, current, preview) => {
            render_buffer_slot(
                mode,
                frame,
                parent,
                buffers.get(&parent.buffer_id),
                context.clone(),
            );
            render_buffer_slot(
                mode,
                frame,
                current,
                buffers.get(&current.buffer_id),
                context.clone(),
            );
            render_buffer_slot(
                mode,
                frame,
                preview,
                buffers.get(&preview.buffer_id),
                context.clone(),
            );

            if let Some(buffer) = buffers.get(&current.buffer_id) {
                let total_width = (preview.x + preview.width).saturating_sub(parent.x);
                let statusline_rect = Rect {
                    x: parent.x.saturating_add(context.horizontal_offset),
                    y: current
                        .y
                        .saturating_add(current.height)
                        .saturating_add(context.vertical_offset),
                    width: total_width,
                    height: 1,
                };

                let mut statusline_vp = current.clone();
                statusline_vp.show_border = context.draw_borders.unwrap_or(preview.show_border);

                statusline::view(
                    buffer,
                    &statusline_vp,
                    frame,
                    statusline_rect,
                    context.is_focused,
                );
            }
        }
        Window::Tasks(vp) => {
            render_buffer_slot(mode, frame, vp, buffers.get(&vp.buffer_id), context.clone());

            if let Some(buffer) = buffers.get(&vp.buffer_id) {
                let statusline_rect = Rect {
                    x: vp.x.saturating_add(context.horizontal_offset),
                    y: vp
                        .y
                        .saturating_add(vp.height)
                        .saturating_add(context.vertical_offset),
                    width: vp.width,
                    height: 1,
                };

                let mut statusline_vp = vp.clone();
                statusline_vp.show_border = context.draw_borders.unwrap_or(vp.show_border);

                statusline::view(
                    buffer,
                    &statusline_vp,
                    frame,
                    statusline_rect,
                    context.is_focused,
                );
            }
        }
    }
}

fn render_buffer_slot(
    mode: &Mode,
    frame: &mut Frame,
    viewport: &ViewPort,
    buffer: Option<&Buffer>,
    context: RenderContext,
) {
    let x = viewport.x.saturating_add(context.horizontal_offset);
    let y = viewport.y.saturating_add(context.vertical_offset);

    let mut effective_vp = if context.is_focused {
        viewport.clone()
    } else {
        ViewPort {
            hide_cursor: true,
            hide_cursor_line: true,
            ..viewport.clone()
        }
    };

    if let Some(true) = context.draw_borders {
        effective_vp.show_border = true;
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
