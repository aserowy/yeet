use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::model::{App, Buffer, SplitFocus, Window};

pub fn render(app: &App, frame: &mut Frame) -> u16 {
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
    ));
    let paragraph =
        Paragraph::new(line).block(Block::default().style(Style::default().bg(Color::Black)));

    frame.render_widget(paragraph, rect);

    1
}

fn tab_spans(
    tabs: &HashMap<usize, Window>,
    buffers: &HashMap<usize, Buffer>,
    current_tab_id: usize,
) -> Vec<Span<'static>> {
    let mut ids: Vec<_> = tabs.keys().copied().collect();
    ids.sort_unstable();

    let mut spans = Vec::new();
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(Color::Gray)));
        }

        let title = tabs
            .get(id)
            .map(|window| tab_title_from_window(window, buffers))
            .unwrap_or_else(|| "(empty)".to_string());

        let label = format!("{}: {}", id, title);
        let style = if *id == current_tab_id {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        spans.push(Span::styled(label, style));
    }

    spans
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

    use super::tab_title_from_window;

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
}
