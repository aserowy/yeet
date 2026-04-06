use std::mem;

use yeet_buffer::model::{viewport::ViewPort, BufferLine, TextBuffer};

use yeet_lua::LuaConfiguration;

use crate::{
    action::Action,
    event::Message,
    model::{App, Buffer, HelpBuffer, SplitFocus, Window},
    task::Task,
    update::{app, hook},
};

const INDEX_CONTENT: &str = include_str!("../../../../docs/help/index.md");
const COMMANDS_CONTENT: &str = include_str!("../../../../docs/help/commands.md");
const CONFIGURATION_CONTENT: &str = include_str!("../../../../docs/help/configuration.md");
const HOOKS_CONTENT: &str = include_str!("../../../../docs/help/hooks.md");
const KEYBINDINGS_CONTENT: &str = include_str!("../../../../docs/help/keybindings.md");
const MODES_CONTENT: &str = include_str!("../../../../docs/help/modes.md");
const THEME_CONTENT: &str = include_str!("../../../../docs/help/theme.md");

struct HelpPage {
    name: &'static str,
    content: &'static str,
}

const HELP_PAGES: &[HelpPage] = &[
    HelpPage {
        name: "index",
        content: INDEX_CONTENT,
    },
    HelpPage {
        name: "commands",
        content: COMMANDS_CONTENT,
    },
    HelpPage {
        name: "configuration",
        content: CONFIGURATION_CONTENT,
    },
    HelpPage {
        name: "hooks",
        content: HOOKS_CONTENT,
    },
    HelpPage {
        name: "keybindings",
        content: KEYBINDINGS_CONTENT,
    },
    HelpPage {
        name: "modes",
        content: MODES_CONTENT,
    },
    HelpPage {
        name: "theme",
        content: THEME_CONTENT,
    },
];

struct TopicMatch {
    content: &'static str,
    line_offset: usize,
}

fn resolve_topic(topic: &str) -> Option<TopicMatch> {
    for page in HELP_PAGES {
        if page.name.eq_ignore_ascii_case(topic) {
            return Some(TopicMatch {
                content: page.content,
                line_offset: 0,
            });
        }
    }

    for page in HELP_PAGES {
        for (line_idx, line) in page.content.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("# ") && trimmed[2..].trim().eq_ignore_ascii_case(topic) {
                return Some(TopicMatch {
                    content: page.content,
                    line_offset: line_idx,
                });
            }

            if trimmed.starts_with("## ") && trimmed[3..].trim().eq_ignore_ascii_case(topic) {
                return Some(TopicMatch {
                    content: page.content,
                    line_offset: line_idx,
                });
            }

            if trimmed.starts_with("### `") && trimmed.ends_with('`') {
                let identifier = &trimmed[5..trimmed.len() - 1];
                if identifier.eq_ignore_ascii_case(topic) {
                    return Some(TopicMatch {
                        content: page.content,
                        line_offset: line_idx,
                    });
                }
            }
        }
    }

    None
}

pub fn open(app: &mut App, lua: Option<&LuaConfiguration>, topic: Option<&str>) -> Vec<Action> {
    let topic_match = match topic {
        Some(t) => match resolve_topic(t) {
            Some(m) => m,
            None => {
                return vec![Action::EmitMessages(vec![Message::Error(format!(
                    "E149: Sorry, no help for {}",
                    t
                ))])];
            }
        },
        None => TopicMatch {
            content: INDEX_CONTENT,
            line_offset: 0,
        },
    };

    let lines: Vec<BufferLine> = topic_match.content.lines().map(BufferLine::from).collect();

    let (window, contents) = match app.current_window_and_contents_mut() {
        Ok(wc) => wc,
        Err(_) => return Vec::new(),
    };

    let buffer_id = app::get_next_buffer_id(contents);
    contents.buffers.insert(
        buffer_id,
        Buffer::Help(HelpBuffer {
            buffer: TextBuffer::from_lines(lines),
        }),
    );

    let mut help_window = Window::Help(ViewPort {
        buffer_id,
        show_border: false,
        wrap: true,
        ..Default::default()
    });

    if let Some(lua) = lua {
        hook::on_window_create(lua, &mut help_window, None);
    }

    let old_window = mem::take(window);
    *window = Window::Horizontal {
        first: Box::new(old_window),
        second: Box::new(help_window),
        focus: SplitFocus::Second,
    };

    if topic_match.line_offset > 0 {
        let vp = match app.current_window_mut() {
            Ok(w) => w.focused_viewport_mut(),
            Err(_) => return Vec::new(),
        };
        vp.cursor.vertical_index = topic_match.line_offset;
        vp.vertical_index = topic_match.line_offset;
    }

    vec![Action::Task(Task::HighlightHelp(
        buffer_id,
        topic_match.content.to_string(),
    ))]
}

pub fn apply_highlighted(app: &mut App, buffer_id: usize, lines: Vec<String>) {
    let buffer = match app.contents.buffers.get_mut(&buffer_id) {
        Some(Buffer::Help(help)) => &mut help.buffer,
        _ => return,
    };

    let highlighted: Vec<BufferLine> = lines
        .iter()
        .flat_map(|l| l.split_terminator('\n').map(BufferLine::from))
        .collect();

    buffer.lines = highlighted;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resolve_topic_index_page() {
        let result = resolve_topic("index");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.line_offset, 0);
        assert!(m.content.contains("Yeet Help"));
    }

    #[test]
    fn resolve_topic_commands_page() {
        let result = resolve_topic("commands");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.line_offset, 0);
        assert!(m.content.contains("# Commands"));
    }

    #[test]
    fn resolve_topic_keybindings_page() {
        let result = resolve_topic("keybindings");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.line_offset, 0);
    }

    #[test]
    fn resolve_topic_entry_identifier() {
        let result = resolve_topic("split");
        assert!(result.is_some());
        let m = result.unwrap();
        assert!(m.line_offset > 0);
        assert!(m.content.contains("# Commands"));
    }

    #[test]
    fn resolve_topic_section_heading() {
        let result = resolve_topic("File Operations");
        assert!(result.is_some());
        let m = result.unwrap();
        assert!(m.line_offset > 0);
    }

    #[test]
    fn resolve_topic_page_title() {
        let result = resolve_topic("Commands");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.line_offset, 0);
    }

    #[test]
    fn resolve_topic_unknown() {
        let result = resolve_topic("nonexistent_topic_xyz");
        assert!(result.is_none());
    }

    #[test]
    fn resolve_topic_help_entry() {
        let result = resolve_topic("help");
        assert!(result.is_some());
    }

    #[test]
    fn open_bare_help_creates_horizontal_split() {
        let mut app = App::default();
        let actions = open(&mut app, None, None);
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            Action::Task(Task::HighlightHelp(_, _))
        ));

        let window = app.current_window().expect("should have current tab");
        assert!(matches!(
            window,
            Window::Horizontal {
                focus: SplitFocus::Second,
                ..
            }
        ));

        match window {
            Window::Horizontal { second, .. } => {
                assert!(matches!(second.as_ref(), Window::Help(_)));
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn open_bare_help_starts_at_line_zero() {
        let mut app = App::default();
        open(&mut app, None, None);

        let window = app.current_window().expect("should have current tab");
        let vp = window.focused_viewport();
        assert_eq!(vp.cursor.vertical_index, 0);
        assert_eq!(vp.vertical_index, 0);
    }

    #[test]
    fn open_help_returns_highlight_task_with_buffer_id() {
        let mut app = App::default();
        let actions = open(&mut app, None, None);
        assert_eq!(actions.len(), 1);

        let window = app.current_window().expect("should have current tab");
        let vp = window.focused_viewport();

        match &actions[0] {
            Action::Task(Task::HighlightHelp(id, content)) => {
                assert_eq!(*id, vp.buffer_id);
                assert!(content.contains("Yeet Help"));
            }
            _ => panic!("expected Action::Task(Task::HighlightHelp)"),
        }
    }

    #[test]
    fn open_help_buffer_contains_raw_content() {
        let mut app = App::default();
        open(&mut app, None, None);

        let window = app.current_window().expect("should have current tab");
        let vp = window.focused_viewport();
        let buffer = app.contents.buffers.get(&vp.buffer_id).unwrap();
        match buffer {
            Buffer::Help(help) => {
                assert!(!help.buffer.lines.is_empty());
                let first_line = help.buffer.lines[0].content.to_stripped_string();
                assert_eq!(first_line, "# Yeet Help");
            }
            _ => panic!("expected Buffer::Help"),
        }
    }

    #[test]
    fn open_help_with_known_topic_creates_split() {
        let mut app = App::default();
        let actions = open(&mut app, None, Some("commands"));
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            Action::Task(Task::HighlightHelp(_, _))
        ));

        let window = app.current_window().expect("should have current tab");
        match window {
            Window::Horizontal { second, .. } => {
                assert!(matches!(second.as_ref(), Window::Help(_)));
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn open_help_with_unknown_topic_returns_error() {
        let mut app = App::default();
        let actions = open(&mut app, None, Some("nonexistent_topic_xyz"));
        assert!(!actions.is_empty());
        assert!(!matches!(actions[0], Action::Task(_)));
    }

    #[test]
    fn open_help_with_topic_scrolls_to_entry() {
        let mut app = App::default();
        open(&mut app, None, Some("split"));

        let window = app.current_window().expect("should have current tab");
        let vp = window.focused_viewport();
        assert!(vp.cursor.vertical_index > 0);
    }

    #[test]
    fn open_help_with_topic_positions_viewport_at_cursor() {
        let mut app = App::default();
        open(&mut app, None, Some("File Operations"));

        let window = app.current_window().expect("should have current tab");
        let vp = window.focused_viewport();
        assert!(vp.cursor.vertical_index > 0);
        assert_eq!(vp.vertical_index, vp.cursor.vertical_index);
    }

    #[test]
    fn resolve_topic_page_name_case_insensitive() {
        let result = resolve_topic("Commands");
        assert!(result.is_some());
        assert_eq!(result.unwrap().line_offset, 0);
    }

    #[test]
    fn resolve_topic_section_heading_case_insensitive() {
        let result = resolve_topic("file operations");
        assert!(result.is_some());
        assert!(result.unwrap().line_offset > 0);
    }

    #[test]
    fn resolve_topic_entry_identifier_case_insensitive() {
        let result = resolve_topic("Split");
        assert!(result.is_some());
        let m = result.unwrap();
        assert!(m.line_offset > 0);
        assert!(m.content.contains("# Commands"));
    }

    #[test]
    fn close_help_restores_directory() {
        let mut app = App::default();
        open(&mut app, None, None);

        let window = app.current_window().expect("should have current tab");
        assert!(matches!(window, Window::Horizontal { .. }));

        let window = app.current_window_mut().expect("should have current tab");
        let (kept, dropped) = mem::take(window).close_focused().ok().unwrap();
        *window = kept;

        assert!(matches!(dropped, Window::Help(_)));
        let window = app.current_window().expect("should have current tab");
        assert!(matches!(window, Window::Directory(..)));
    }
}
