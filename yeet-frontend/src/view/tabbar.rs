use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{model::{App, Buffer, SplitFocus, Window}, theme::Theme};

const TAB_WIDTH: usize = 28;

pub fn render(app: &App, theme: &Theme, frame: &mut Frame) -> u16 {
    if app.tabs.len() <= 1 {
        return 0;
    }

    let width = frame.area().width;
    let rect = Rect {
        x: 0,
        y: 0,
        width,
        height: 1,
    };

    let line = Line::from(tab_spans(
        &app.tabs,
        &app.contents.buffers,
        app.current_tab_id,
        width as usize,
        theme,
    ));
    let paragraph = Paragraph::new(line).block(Block::default());

    frame.render_widget(paragraph, rect);

    1
}

fn tab_spans(
    tabs: &HashMap<usize, Window>,
    buffers: &HashMap<usize, Buffer>,
    current_tab_id: usize,
    total_width: usize,
    theme: &Theme,
) -> Vec<Span<'static>> {
    use crate::theme::tokens;

    let mut ids: Vec<_> = tabs.keys().copied().collect();
    ids.sort_unstable();

    let mut spans = Vec::new();
    for id in ids.iter() {
        let title = tabs
            .get(id)
            .map(|window| tab_title_from_window(window, buffers))
            .unwrap_or_else(|| "(empty)".to_string());

        if *id == current_tab_id {
            let label = format_tab_label(*id, &title);
            spans.push(Span::styled(
                label,
                theme
                    .style_fg_bg(tokens::TABBAR_ACTIVE_FG, tokens::TABBAR_ACTIVE_BG)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            let label = format_tab_label(*id, &title);
            spans.push(Span::styled(
                label,
                theme.style_fg_bg(tokens::TABBAR_INACTIVE_FG, tokens::TABBAR_INACTIVE_BG),
            ));
        }
    }

    let used = ids.len().saturating_mul(TAB_WIDTH);
    let remaining = total_width.saturating_sub(used);
    if remaining > 0 {
        spans.push(Span::styled(
            " ".repeat(remaining),
            theme.style_bg(tokens::TABBAR_BG),
        ));
    }

    spans
}

fn format_tab_label(index: usize, title: &str) -> String {
    let index_str = format!(" {} ", index);
    let available = TAB_WIDTH.saturating_sub(index_str.len() + 1);
    let mut title = title.to_string();
    if title.len() > available {
        title.truncate(available);
    }
    let padding = available.saturating_sub(title.len());
    let left = padding / 2;
    let right = padding - left;
    let prefix = format!("{}{}", index_str, " ".repeat(left));
    let suffix = format!("{} ", " ".repeat(right));

    format!("{}{}{}", prefix, title, suffix)
}

fn tab_title_from_window(window: &Window, buffers: &HashMap<usize, Buffer>) -> String {
    match window {
        Window::Horizontal {
            first,
            second,
            focus,
        }
        | Window::Vertical {
            first,
            second,
            focus,
        } => match focus {
            SplitFocus::First => tab_title_from_window(first, buffers),
            SplitFocus::Second => tab_title_from_window(second, buffers),
        },
        Window::Directory(_, current, _) => {
            if let Some(Buffer::Directory(dir)) = buffers.get(&current.buffer_id) {
                if let Some(path) = dir.resolve_path() {
                    return path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("(empty)")
                        .to_string();
                }
            }
            "(empty)".to_string()
        }
        Window::Tasks(_) => "Tasks".to_string(),
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, path::PathBuf};

    use yeet_buffer::model::viewport::ViewPort;

    use crate::model::{Buffer, DirectoryBuffer, SplitFocus, Window};

    use super::{format_tab_label, tab_title_from_window, TAB_WIDTH};

    #[test]
    fn tab_title_uses_directory_folder_name() {
        let mut buffers = HashMap::new();
        buffers.insert(
            1,
            Buffer::Directory(DirectoryBuffer {
                path: PathBuf::from("/tmp/project/src"),
                ..Default::default()
            }),
        );

        let window = Window::Directory(
            ViewPort::default(),
            ViewPort {
                buffer_id: 1,
                ..Default::default()
            },
            ViewPort::default(),
        );
        let title = tab_title_from_window(&window, &buffers);
        assert_eq!(title, "src");
    }

    #[test]
    fn tab_title_uses_focused_child_in_split() {
        let mut buffers = HashMap::new();
        buffers.insert(
            1,
            Buffer::Directory(DirectoryBuffer {
                path: PathBuf::from("/tmp/project/current"),
                ..Default::default()
            }),
        );

        let focused = Window::Directory(
            ViewPort::default(),
            ViewPort {
                buffer_id: 1,
                ..Default::default()
            },
            ViewPort::default(),
        );
        let window = Window::Horizontal {
            first: Box::new(focused),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::First,
        };

        let title = tab_title_from_window(&window, &buffers);
        assert_eq!(title, "current");
    }

    #[test]
    fn tab_title_uses_tasks_when_focused() {
        let buffers = HashMap::new();
        let window = Window::Tasks(ViewPort::default());

        let title = tab_title_from_window(&window, &buffers);
        assert_eq!(title, "Tasks");
    }

    #[test]
    fn format_tab_label_centers_and_truncates() {
        let label = format_tab_label(2, "src");
        assert_eq!(label.len(), TAB_WIDTH);
        assert!(label.starts_with(" 2 "));
        assert!(label.ends_with(' '));

        let long = format_tab_label(10, "this-title-is-way-too-long-to-fit");
        assert_eq!(long.len(), TAB_WIDTH);
        assert!(long.starts_with(" 10 "));
        assert!(long.ends_with(' '));
    }
}
