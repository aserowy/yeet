use std::mem;
use std::path::PathBuf;

use yeet_buffer::model::{viewport::ViewPort, BufferLine, TextBuffer};

use yeet_lua::LuaConfiguration;

use crate::{
    action::Action,
    event::{LogSeverity, Message},
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
const PLUGINS_CONTENT: &str = include_str!("../../../../docs/help/plugins.md");
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
        name: "plugins",
        content: PLUGINS_CONTENT,
    },
    HelpPage {
        name: "theme",
        content: THEME_CONTENT,
    },
];

struct PluginHelpPage {
    name: String,
    content: String,
}

enum TopicMatch {
    Core {
        content: &'static str,
        line_offset: usize,
    },
    Plugin {
        content: String,
        line_offset: usize,
    },
}

impl TopicMatch {
    fn content(&self) -> &str {
        match self {
            TopicMatch::Core { content, .. } => content,
            TopicMatch::Plugin { content, .. } => content.as_str(),
        }
    }

    fn line_offset(&self) -> usize {
        match self {
            TopicMatch::Core { line_offset, .. } => *line_offset,
            TopicMatch::Plugin { line_offset, .. } => *line_offset,
        }
    }
}

fn resolve_core_topic(topic: &str) -> Option<TopicMatch> {
    for page in HELP_PAGES {
        if page.name.eq_ignore_ascii_case(topic) {
            return Some(TopicMatch::Core {
                content: page.content,
                line_offset: 0,
            });
        }
    }

    for page in HELP_PAGES {
        if let Some(line_offset) = find_heading_match(page.content, topic) {
            return Some(TopicMatch::Core {
                content: page.content,
                line_offset,
            });
        }
    }

    None
}

fn resolve_plugin_topic(topic: &str, plugin_pages: &[PluginHelpPage]) -> Option<TopicMatch> {
    for page in plugin_pages {
        if page.name.eq_ignore_ascii_case(topic) {
            return Some(TopicMatch::Plugin {
                content: page.content.clone(),
                line_offset: 0,
            });
        }
    }

    for page in plugin_pages {
        if let Some(line_offset) = find_heading_match(&page.content, topic) {
            return Some(TopicMatch::Plugin {
                content: page.content.clone(),
                line_offset,
            });
        }
    }

    None
}

fn find_heading_match(content: &str, topic: &str) -> Option<usize> {
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("# ") && trimmed[2..].trim().eq_ignore_ascii_case(topic) {
            return Some(line_idx);
        }

        if trimmed.starts_with("## ") && trimmed[3..].trim().eq_ignore_ascii_case(topic) {
            return Some(line_idx);
        }

        if trimmed.starts_with("### `") && trimmed.ends_with('`') {
            let identifier = &trimmed[5..trimmed.len() - 1];
            if identifier.eq_ignore_ascii_case(topic) {
                return Some(line_idx);
            }
        }
    }
    None
}

fn resolve_topic(topic: &str, plugin_pages: &[PluginHelpPage]) -> Option<TopicMatch> {
    // Core pages take priority
    if let Some(m) = resolve_core_topic(topic) {
        return Some(m);
    }

    // Fall back to plugin pages
    resolve_plugin_topic(topic, plugin_pages)
}

fn discover_plugin_help_pages(lua: &LuaConfiguration) -> Vec<PluginHelpPage> {
    let mut pages = Vec::new();

    let data_path = match yeet_lua::read_plugin_data_path(lua) {
        Some(p) => p,
        None => return pages,
    };

    let specs = yeet_lua::read_plugin_specs(lua);
    for spec in &specs {
        let storage_path = match yeet_plugin::url_to_storage_path(&spec.url) {
            Some(p) => p,
            None => continue,
        };

        let plugin_dir = data_path.join(storage_path);
        let help_dir = plugin_dir.join("docs").join("help");

        if !help_dir.is_dir() {
            continue;
        }

        let entries = match std::fs::read_dir(&help_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let name = match path.file_stem().and_then(|s| s.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            pages.push(PluginHelpPage { name, content });
        }
    }

    pages
}

fn discover_plugin_help_pages_from_paths(plugin_dirs: &[PathBuf]) -> Vec<PluginHelpPage> {
    let mut pages = Vec::new();

    for plugin_dir in plugin_dirs {
        let help_dir = plugin_dir.join("docs").join("help");

        if !help_dir.is_dir() {
            continue;
        }

        let entries = match std::fs::read_dir(&help_dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }

            let name = match path.file_stem().and_then(|s| s.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            pages.push(PluginHelpPage { name, content });
        }
    }

    pages
}

pub fn open(app: &mut App, lua: Option<&LuaConfiguration>, topic: Option<&str>) -> Vec<Action> {
    let plugin_pages = lua
        .map(|l| discover_plugin_help_pages(l))
        .unwrap_or_default();

    let topic_match = match topic {
        Some(t) => match resolve_topic(t, &plugin_pages) {
            Some(m) => m,
            None => {
                return vec![Action::EmitMessages(vec![Message::Log(
                    LogSeverity::Error,
                    format!("E149: Sorry, no help for {}", t),
                )])];
            }
        },
        None => TopicMatch::Core {
            content: INDEX_CONTENT,
            line_offset: 0,
        },
    };

    let content_str = topic_match.content();

    let lines: Vec<BufferLine> = content_str
        .lines()
        .map(|l| {
            let mut line = BufferLine::from(l);
            if let Some(lua) = lua {
                yeet_lua::invoke_on_bufferline_mutate(
                    lua,
                    &mut line,
                    yeet_lua::BufferType::Help,
                    None,
                );
            }
            line
        })
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

    let line_offset = topic_match.line_offset();
    if line_offset > 0 {
        let vp = match app.current_window_mut() {
            Ok(w) => w.focused_viewport_mut(),
            Err(_) => return Vec::new(),
        };
        vp.cursor.vertical_index = line_offset;
        vp.vertical_index = line_offset;
    }

    vec![Action::Task(Task::HighlightHelp(
        buffer_id,
        content_str.to_string(),
    ))]
}

pub fn apply_highlighted(
    app: &mut App,
    lua: Option<&LuaConfiguration>,
    buffer_id: usize,
    lines: Vec<String>,
) {
    let buffer = match app.contents.buffers.get_mut(&buffer_id) {
        Some(Buffer::Help(help)) => &mut help.buffer,
        _ => return,
    };

    let highlighted: Vec<BufferLine> = lines
        .iter()
        .flat_map(|l| {
            l.split_terminator('\n').map(|s| {
                let mut line = BufferLine::from(s);
                if let Some(lua) = lua {
                    yeet_lua::invoke_on_bufferline_mutate(
                        lua,
                        &mut line,
                        yeet_lua::BufferType::Help,
                        None,
                    );
                }
                line
            })
        })
        .collect();

    buffer.lines = highlighted;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resolve_core_topic_index_page() {
        let result = resolve_core_topic("index");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.line_offset(), 0);
        assert!(m.content().contains("Yeet Help"));
    }

    #[test]
    fn resolve_core_topic_commands_page() {
        let result = resolve_core_topic("commands");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.line_offset(), 0);
        assert!(m.content().contains("# Commands"));
    }

    #[test]
    fn resolve_core_topic_keybindings_page() {
        let result = resolve_core_topic("keybindings");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.line_offset(), 0);
    }

    #[test]
    fn resolve_core_topic_entry_identifier() {
        let result = resolve_core_topic("split");
        assert!(result.is_some());
        let m = result.unwrap();
        assert!(m.line_offset() > 0);
        assert!(m.content().contains("# Commands"));
    }

    #[test]
    fn resolve_core_topic_section_heading() {
        let result = resolve_core_topic("File Operations");
        assert!(result.is_some());
        let m = result.unwrap();
        assert!(m.line_offset() > 0);
    }

    #[test]
    fn resolve_core_topic_page_title() {
        let result = resolve_core_topic("Commands");
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.line_offset(), 0);
    }

    #[test]
    fn resolve_core_topic_unknown() {
        let result = resolve_core_topic("nonexistent_topic_xyz");
        assert!(result.is_none());
    }

    #[test]
    fn resolve_core_topic_help_entry() {
        let result = resolve_core_topic("help");
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
    fn resolve_core_topic_page_name_case_insensitive() {
        let result = resolve_core_topic("Commands");
        assert!(result.is_some());
        assert_eq!(result.unwrap().line_offset(), 0);
    }

    #[test]
    fn resolve_core_topic_section_heading_case_insensitive() {
        let result = resolve_core_topic("file operations");
        assert!(result.is_some());
        assert!(result.unwrap().line_offset() > 0);
    }

    #[test]
    fn resolve_core_topic_entry_identifier_case_insensitive() {
        let result = resolve_core_topic("Split");
        assert!(result.is_some());
        let m = result.unwrap();
        assert!(m.line_offset() > 0);
        assert!(m.content().contains("# Commands"));
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

    #[test]
    fn plugin_help_page_by_name() {
        let pages = vec![PluginHelpPage {
            name: "my-plugin".to_string(),
            content: "# My Plugin\n\nHelp content here.".to_string(),
        }];
        let result = resolve_topic("my-plugin", &pages);
        assert!(result.is_some());
        let m = result.unwrap();
        assert_eq!(m.line_offset(), 0);
        assert!(m.content().contains("My Plugin"));
    }

    #[test]
    fn plugin_help_page_heading_search() {
        let pages = vec![PluginHelpPage {
            name: "my-plugin".to_string(),
            content: "# My Plugin\n\n## Configuration\n\nSome config.".to_string(),
        }];
        let result = resolve_topic("Configuration", &pages);
        assert!(result.is_some());
        let m = result.unwrap();
        assert!(m.line_offset() > 0);
    }

    #[test]
    fn core_takes_priority_over_plugin() {
        let pages = vec![PluginHelpPage {
            name: "theme".to_string(),
            content: "# Plugin Theme\n\nPlugin theme docs.".to_string(),
        }];
        let result = resolve_topic("theme", &pages);
        assert!(result.is_some());
        // Should match the core theme page, not the plugin one
        assert!(result.unwrap().content().contains("y.theme"));
    }

    #[test]
    fn plugin_page_found_when_no_core_match() {
        let pages = vec![PluginHelpPage {
            name: "directory-icons".to_string(),
            content: "# Directory Icons\n\nIcon plugin docs.".to_string(),
        }];
        let result = resolve_topic("directory-icons", &pages);
        assert!(result.is_some());
        assert!(result.unwrap().content().contains("Icon plugin docs"));
    }

    #[test]
    fn no_plugin_pages_falls_through() {
        let pages: Vec<PluginHelpPage> = Vec::new();
        let result = resolve_topic("nonexistent_topic_xyz", &pages);
        assert!(result.is_none());
    }

    #[test]
    fn discover_pages_from_empty_dirs() {
        let dir = tempfile::TempDir::new().unwrap();
        let pages = discover_plugin_help_pages_from_paths(&[dir.path().to_path_buf()]);
        assert!(pages.is_empty());
    }

    #[test]
    fn discover_pages_from_plugin_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        let help_dir = dir.path().join("docs").join("help");
        std::fs::create_dir_all(&help_dir).unwrap();
        std::fs::write(
            help_dir.join("my-plugin.md"),
            "# My Plugin\n\nHelp content.",
        )
        .unwrap();
        std::fs::write(help_dir.join("not-markdown.txt"), "ignored").unwrap();

        let pages = discover_plugin_help_pages_from_paths(&[dir.path().to_path_buf()]);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].name, "my-plugin");
        assert!(pages[0].content.contains("My Plugin"));
    }

    #[test]
    fn discover_pages_ignores_missing_help_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        // No help/ subdirectory
        let pages = discover_plugin_help_pages_from_paths(&[dir.path().to_path_buf()]);
        assert!(pages.is_empty());
    }
}
