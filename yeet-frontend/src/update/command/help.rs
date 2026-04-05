use std::mem;

use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};
use yeet_buffer::model::{viewport::ViewPort, BufferLine, TextBuffer};

use crate::{
    action::Action,
    event::Message,
    model::{App, Buffer, HelpBuffer, SplitFocus, Window},
    update::app,
};

const INDEX_CONTENT: &str = include_str!("../../../../docs/help/index.md");
const COMMANDS_CONTENT: &str = include_str!("../../../../docs/help/commands.md");
const KEYBINDINGS_CONTENT: &str = include_str!("../../../../docs/help/keybindings.md");

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
        name: "keybindings",
        content: KEYBINDINGS_CONTENT,
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

fn highlight_markdown(content: &str) -> Vec<String> {
    let syntaxes = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let theme = &theme_set.themes["base16-ocean.dark"];

    let syntax = syntaxes
        .find_syntax_by_extension("md")
        .unwrap_or_else(|| syntaxes.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut result = Vec::new();

    for line in LinesWithEndings::from(content) {
        let highlighted = match highlighter.highlight_line(line, &syntaxes) {
            Ok(ranges) => as_24_bit_terminal_escaped(&ranges[..], false),
            Err(_) => line.to_string(),
        };
        result.push(highlighted);
    }

    result
}

pub fn open(app: &mut App, topic: Option<&str>) -> Vec<Action> {
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

    let highlighted_lines = highlight_markdown(topic_match.content);
    let lines: Vec<BufferLine> = highlighted_lines
        .iter()
        .map(|l| BufferLine::from(l.as_str()))
        .collect();

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

    let help_viewport = ViewPort {
        buffer_id,
        show_border: false,
        ..Default::default()
    };

    let old_window = mem::take(window);
    *window = Window::Horizontal {
        first: Box::new(old_window),
        second: Box::new(Window::Help(help_viewport)),
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

    Vec::new()
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
        let actions = open(&mut app, None);
        assert!(actions.is_empty());

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
        open(&mut app, None);

        let window = app.current_window().expect("should have current tab");
        let vp = window.focused_viewport();
        assert_eq!(vp.cursor.vertical_index, 0);
        assert_eq!(vp.vertical_index, 0);
    }

    #[test]
    fn open_help_with_known_topic_creates_split() {
        let mut app = App::default();
        let actions = open(&mut app, Some("commands"));
        assert!(actions.is_empty());

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
        let actions = open(&mut app, Some("nonexistent_topic_xyz"));
        assert!(!actions.is_empty());
    }

    #[test]
    fn open_help_with_topic_scrolls_to_entry() {
        let mut app = App::default();
        open(&mut app, Some("split"));

        let window = app.current_window().expect("should have current tab");
        let vp = window.focused_viewport();
        assert!(vp.cursor.vertical_index > 0);
    }

    #[test]
    fn open_help_with_topic_positions_viewport_at_cursor() {
        let mut app = App::default();
        open(&mut app, Some("File Operations"));

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
        open(&mut app, None);

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
