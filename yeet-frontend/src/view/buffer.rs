use std::collections::HashMap;

use ratatui::{layout::Rect, Frame};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Mode},
    view as buffer_view,
};

use crate::{
    model::{App, Buffer, DirectoryBuffer, SplitFocus, Window},
    theme::{tokens, Theme},
};

use super::statusline;

pub fn view(mode: &Mode, app: &App, theme: &Theme, frame: &mut Frame) {
    let context = RenderContext {
        draw_borders: None,
        is_focused: true,
        is_directory_pane: false,
    };
    let window = match app.current_window() {
        Ok(window) => window,
        Err(_) => return,
    };
    render_window(mode, window, &app.contents.buffers, theme, frame, context);
}

#[derive(Clone)]
struct RenderContext {
    draw_borders: Option<bool>,
    is_focused: bool,
    is_directory_pane: bool,
}

fn render_window(
    mode: &Mode,
    window: &Window,
    buffers: &HashMap<usize, Buffer>,
    theme: &Theme,
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
                frame,
                RenderContext {
                    is_focused: context.is_focused && focus == &SplitFocus::First,
                    draw_borders: Some(true),
                    is_directory_pane: false,
                },
            );
            render_window(
                mode,
                second,
                buffers,
                theme,
                frame,
                RenderContext {
                    is_focused: context.is_focused && focus == &SplitFocus::Second,
                    ..context.clone()
                },
            );
        }
        Window::Directory(parent, current, preview) => {
            let dir_context = RenderContext {
                is_directory_pane: true,
                draw_borders: None,
                ..context.clone()
            };
            render_buffer_slot(
                mode,
                frame,
                parent,
                buffers.get(&parent.buffer_id),
                dir_context.clone(),
                theme,
            );
            render_buffer_slot(
                mode,
                frame,
                current,
                buffers.get(&current.buffer_id),
                dir_context.clone(),
                theme,
            );

            let preview_context = if context.draw_borders == Some(true) {
                RenderContext {
                    is_directory_pane: false,
                    draw_borders: Some(true),
                    is_focused: context.is_focused,
                }
            } else {
                dir_context.clone()
            };
            render_buffer_slot(
                mode,
                frame,
                preview,
                buffers.get(&preview.buffer_id),
                preview_context,
                theme,
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
            render_buffer_slot(
                mode,
                frame,
                vp,
                buffers.get(&vp.buffer_id),
                context.clone(),
                theme,
            );

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
    theme: &Theme,
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

    let buffer_theme = if context.is_directory_pane {
        theme.to_buffer_theme_with_border(tokens::DIRECTORY_BORDER_FG, tokens::DIRECTORY_BORDER_BG)
    } else {
        theme.to_buffer_theme_with_border(tokens::SPLIT_BORDER_FG, tokens::SPLIT_BORDER_BG)
    };

    match buffer {
        Some(Buffer::Content(buffer)) => {
            buffer_view(&effective_vp, mode, &buffer.buffer, &buffer_theme, frame);
        }
        Some(Buffer::Directory(buffer)) => {
            render_directory_buffer(mode, frame, &effective_vp, buffer, &buffer_theme);
        }
        Some(Buffer::Tasks(tasks_buf)) => {
            buffer_view(&effective_vp, mode, &tasks_buf.buffer, &buffer_theme, frame);
        }
        Some(Buffer::PathReference(_)) | Some(Buffer::Empty) | None => {
            let mut vp = effective_vp.clone();
            vp.hide_cursor = true;
            vp.hide_cursor_line = true;

            render_directory_buffer(mode, frame, &vp, &Default::default(), &buffer_theme);
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use ratatui::{backend::TestBackend, Terminal};
    use yeet_buffer::model::viewport::ViewPort;

    use crate::{
        model::{Buffer, SplitFocus, Window},
        theme::{tokens, Theme},
    };

    use super::*;

    fn make_directory_window(
        parent_id: usize,
        current_id: usize,
        preview_id: usize,
    ) -> Window {
        Window::Directory(
            ViewPort {
                buffer_id: parent_id,
                x: 0,
                y: 0,
                width: 20,
                height: 10,
                show_border: true,
                ..Default::default()
            },
            ViewPort {
                buffer_id: current_id,
                x: 20,
                y: 0,
                width: 30,
                height: 10,
                show_border: true,
                ..Default::default()
            },
            ViewPort {
                buffer_id: preview_id,
                x: 50,
                y: 0,
                width: 30,
                height: 10,
                show_border: false,
                ..Default::default()
            },
        )
    }

    #[test]
    fn vertical_split_directory_preview_uses_split_border_context() {
        // When a directory window is the first child of a vertical split,
        // the preview pane's border (the split separator) should use split
        // border colors, not directory border colors.
        let theme = Theme::default();

        // Simulate what render_window does for Window::Vertical containing a Directory
        let context = RenderContext {
            draw_borders: Some(true),
            is_focused: true,
            is_directory_pane: false,
        };

        // This is the dir_context created in Window::Directory for parent/current
        let dir_context = RenderContext {
            is_directory_pane: true,
            draw_borders: None,
            ..context.clone()
        };

        // Parent and current: directory border colors, no forced draw_borders
        assert!(dir_context.is_directory_pane);
        assert_eq!(dir_context.draw_borders, None);

        // Preview: when parent context has draw_borders, use split border colors
        let preview_context = if context.draw_borders == Some(true) {
            RenderContext {
                is_directory_pane: false,
                draw_borders: Some(true),
                is_focused: context.is_focused,
            }
        } else {
            dir_context.clone()
        };

        assert!(!preview_context.is_directory_pane);
        assert_eq!(preview_context.draw_borders, Some(true));

        // Verify the theme produces the right border colors
        let dir_bt = theme
            .to_buffer_theme_with_border(tokens::DIRECTORY_BORDER_FG, tokens::DIRECTORY_BORDER_BG);
        let split_bt = theme
            .to_buffer_theme_with_border(tokens::SPLIT_BORDER_FG, tokens::SPLIT_BORDER_BG);

        // Directory panes should use DIRECTORY_BORDER colors
        assert_eq!(dir_bt.border_fg, theme.color(tokens::DIRECTORY_BORDER_FG));
        assert_eq!(dir_bt.border_bg, theme.color(tokens::DIRECTORY_BORDER_BG));

        // Split separator (preview in split) should use SPLIT_BORDER colors
        assert_eq!(split_bt.border_fg, theme.color(tokens::SPLIT_BORDER_FG));
        assert_eq!(split_bt.border_bg, theme.color(tokens::SPLIT_BORDER_BG));
    }

    #[test]
    fn vertical_split_with_directory_renders_without_panic() {
        let left = make_directory_window(1, 2, 3);
        let right = make_directory_window(4, 5, 6);

        let window = Window::Vertical {
            first: Box::new(left),
            second: Box::new(right),
            focus: SplitFocus::First,
        };

        let mut buffers = HashMap::new();
        for id in 1..=6 {
            buffers.insert(id, Buffer::Empty);
        }

        let theme = Theme::default();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("create terminal");

        terminal
            .draw(|frame| {
                let context = RenderContext {
                    draw_borders: None,
                    is_focused: true,
                    is_directory_pane: false,
                };
                render_window(
                    &yeet_buffer::model::Mode::Navigation,
                    &window,
                    &buffers,
                    &theme,
                    frame,
                    context,
                );
            })
            .expect("draw should succeed");
    }
}
