use std::collections::HashMap;

use ratatui::{layout::Rect, Frame};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Mode},
    view_themed as buffer_view,
};

use crate::{
    model::{App, Buffer, DirectoryBuffer, SplitFocus, Window},
    theme::Theme,
};

use super::statusline;

pub fn view(mode: &Mode, app: &App, theme: &Theme, frame: &mut Frame) {
    let context = RenderContext {
        draw_borders: None,
        is_focused: true,
    };
    let buffer_theme = theme.to_buffer_theme();

    let window = match app.current_window() {
        Ok(window) => window,
        Err(_) => return,
    };
    render_window(mode, window, &app.contents.buffers, theme, &buffer_theme, frame, context);
}

#[derive(Clone)]
struct RenderContext {
    draw_borders: Option<bool>,
    is_focused: bool,
}

fn render_window(
    mode: &Mode,
    window: &Window,
    buffers: &HashMap<usize, Buffer>,
    theme: &Theme,
    buffer_theme: &yeet_buffer::BufferTheme,
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
                theme,
                buffer_theme,
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
                theme,
                buffer_theme,
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
                theme,
                buffer_theme,
                frame,
                RenderContext {
                    is_focused: context.is_focused && focus == &SplitFocus::First,
                    draw_borders: Some(true),
                },
            );
            render_window(
                mode,
                second,
                buffers,
                theme,
                buffer_theme,
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
                buffer_theme,
            );
            render_buffer_slot(
                mode,
                frame,
                current,
                buffers.get(&current.buffer_id),
                context.clone(),
                buffer_theme,
            );
            render_buffer_slot(
                mode,
                frame,
                preview,
                buffers.get(&preview.buffer_id),
                context.clone(),
                buffer_theme,
            );

            if let Some(buffer) = buffers.get(&current.buffer_id) {
                let total_width = (preview.x + preview.width).saturating_sub(parent.x);
                let statusline_rect = Rect {
                    x: parent.x,
                    y: current.y.saturating_add(current.height),
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
                    theme,
                );
            }
        }
        Window::Tasks(vp) => {
            render_buffer_slot(mode, frame, vp, buffers.get(&vp.buffer_id), context.clone(), buffer_theme);

            if let Some(buffer) = buffers.get(&vp.buffer_id) {
                let statusline_rect = Rect {
                    x: vp.x,
                    y: vp.y.saturating_add(vp.height),
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
                    theme,
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
    buffer_theme: &yeet_buffer::BufferTheme,
) {
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
            buffer_view(&effective_vp, mode, &buffer.buffer, buffer_theme, frame);
        }
        Some(Buffer::Directory(buffer)) => {
            render_directory_buffer(mode, frame, &effective_vp, buffer, buffer_theme);
        }
        Some(Buffer::Tasks(tasks_buf)) => {
            buffer_view(&effective_vp, mode, &tasks_buf.buffer, buffer_theme, frame);
        }
        Some(Buffer::PathReference(_)) | Some(Buffer::Empty) | None => {
            let mut vp = effective_vp.clone();
            vp.hide_cursor = true;
            vp.hide_cursor_line = true;

            render_directory_buffer(mode, frame, &vp, &Default::default(), buffer_theme);
        }
        Some(Buffer::Image(buffer)) => {
            let rect = Rect {
                x: viewport.x,
                y: viewport.y,
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
    buffer_theme: &yeet_buffer::BufferTheme,
) {
    let mut viewport = viewport.clone();
    yeet_buffer::update_viewport_by_cursor(&mut viewport, &buffer.buffer);

    buffer_view(&viewport, mode, &buffer.buffer, buffer_theme, frame);
}
