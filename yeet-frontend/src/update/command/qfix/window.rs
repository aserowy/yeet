use std::{collections::HashMap, mem, path::Path};

use yeet_buffer::model::{viewport::ViewPort, BufferLine, TextBuffer};
use yeet_lua::LuaConfiguration;

use crate::{
    action::Action,
    model::{qfix::QuickFix, App, Buffer, Contents, QuickFixBuffer, SplitFocus, Window},
    update::{app, hook},
};

pub fn open(app: &mut App, lua: Option<&LuaConfiguration>, qfix: &QuickFix) -> Vec<Action> {
    let (window, contents) = match app.current_window_and_contents_mut() {
        Ok(window) => window,
        Err(_) => return Vec::new(),
    };
    if focus_quickfix(window) {
        return Vec::new();
    }

    let lines = build_qfix_lines(qfix, lua);
    let buffer_id = app::get_next_buffer_id(contents);
    contents.buffers.insert(
        buffer_id,
        Buffer::QuickFix(QuickFixBuffer {
            buffer: TextBuffer::from_lines(lines),
        }),
    );

    let mut qfix_window = Window::QuickFix(ViewPort {
        buffer_id,
        show_border: false,
        ..Default::default()
    });

    if let Some(lua) = lua {
        hook::on_window_create(lua, &mut qfix_window, None);
    }

    let old_window = mem::take(window);
    *window = Window::Horizontal {
        first: Box::new(old_window),
        second: Box::new(qfix_window),
        focus: SplitFocus::Second,
    };

    Vec::new()
}

fn focus_quickfix(window: &mut Window) -> bool {
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
        } => {
            if second.contains_quickfix() {
                *focus = SplitFocus::Second;
                focus_quickfix(second)
            } else if first.contains_quickfix() {
                *focus = SplitFocus::First;
                focus_quickfix(first)
            } else {
                false
            }
        }
        Window::QuickFix(_) => true,
        Window::Tasks(_) | Window::Help(_) | Window::Directory(_, _, _) => false,
    }
}

pub fn focus_nearest_directory(window: &mut Window) -> bool {
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
        } => {
            let focused_is_qfix = match focus {
                SplitFocus::First => matches!(first.as_ref(), Window::QuickFix(_)),
                SplitFocus::Second => matches!(second.as_ref(), Window::QuickFix(_)),
            };

            if focused_is_qfix {
                *focus = match focus {
                    SplitFocus::First => SplitFocus::Second,
                    SplitFocus::Second => SplitFocus::First,
                };
                true
            } else {
                focus_nearest_directory(first) || focus_nearest_directory(second)
            }
        }
        Window::QuickFix(_) | Window::Tasks(_) | Window::Help(_) | Window::Directory(_, _, _) => {
            false
        }
    }
}

pub fn build_qfix_lines(qfix: &QuickFix, lua: Option<&LuaConfiguration>) -> Vec<BufferLine> {
    let max_width = (qfix.entries.len() + 1).to_string().len();
    qfix.entries
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let status = if path.exists() { "" } else { " (removed)" };
            let formatted = format!("{:>max_width$} {}{}", i + 1, path.display(), status);
            let mut line = if i == qfix.current_index {
                BufferLine::from(&format!("\x1b[1m{}\x1b[0m", formatted))
            } else {
                BufferLine::from(&formatted)
            };
            if let Some(lua) = lua {
                yeet_lua::invoke_on_bufferline_mutate(lua, &mut line, "quickfix", Path::new(""));
            }
            line
        })
        .collect()
}

pub fn refresh_quickfix_buffer(
    tabs: &mut HashMap<usize, Window>,
    contents: &mut Contents,
    qfix: &QuickFix,
    lua: Option<&LuaConfiguration>,
) {
    for window in tabs.values_mut() {
        refresh_quickfix_buffer_in_window(window, contents, qfix, lua);
    }
}

fn refresh_quickfix_buffer_in_window(
    window: &mut Window,
    contents: &mut Contents,
    qfix: &QuickFix,
    lua: Option<&LuaConfiguration>,
) {
    let vp = match find_quickfix_viewport_mut(window) {
        Some(vp) => vp,
        None => return,
    };

    let buffer_id = vp.buffer_id;
    if let Some(Buffer::QuickFix(qfix_buffer)) = contents.buffers.get_mut(&buffer_id) {
        qfix_buffer.buffer.lines = build_qfix_lines(qfix, lua);
        let line_count = qfix_buffer.buffer.lines.len();

        if vp.cursor.vertical_index >= line_count {
            vp.cursor.vertical_index = line_count.saturating_sub(1);
        }
    }
}

pub fn find_quickfix_viewport_mut(window: &mut Window) -> Option<&mut ViewPort> {
    match window {
        Window::Horizontal { first, second, .. } | Window::Vertical { first, second, .. } => {
            find_quickfix_viewport_mut(first).or_else(|| find_quickfix_viewport_mut(second))
        }
        Window::QuickFix(vp) => Some(vp),
        Window::Tasks(_) | Window::Help(_) | Window::Directory(_, _, _) => None,
    }
}

pub fn remove_entry(
    app: &mut App,
    lua: Option<&LuaConfiguration>,
    qfix: &mut QuickFix,
    cursor_index: usize,
) -> Vec<Action> {
    if qfix.entries.is_empty() {
        return Vec::new();
    }

    let cursor_index = cursor_index.min(qfix.entries.len().saturating_sub(1));
    let removed_path = qfix.entries.remove(cursor_index);

    match cursor_index.cmp(&qfix.current_index) {
        std::cmp::Ordering::Less => {
            qfix.current_index = qfix.current_index.saturating_sub(1);
        }
        std::cmp::Ordering::Equal => {
            if qfix.entries.is_empty() {
                qfix.current_index = 0;
            } else {
                qfix.current_index = qfix.current_index.min(qfix.entries.len().saturating_sub(1));
            }
        }
        std::cmp::Ordering::Greater => {}
    }

    use crate::model::qfix::QFIX_SIGN_ID;
    use crate::update::sign;
    sign::unset_sign_for_paths(
        app.contents.buffers.values_mut().collect(),
        vec![removed_path],
        QFIX_SIGN_ID,
    );

    refresh_quickfix_buffer(&mut app.tabs, &mut app.contents, qfix, lua);

    Vec::new()
}

pub fn find_nearest_directory_in_sibling(window: &Window) -> Option<(usize, usize, usize)> {
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
        } => {
            let (focused_child, sibling) = match focus {
                SplitFocus::First => (first.as_ref(), second.as_ref()),
                SplitFocus::Second => (second.as_ref(), first.as_ref()),
            };

            if matches!(focused_child, Window::QuickFix(_)) {
                return find_first_directory_by_focus(sibling);
            }

            find_nearest_directory_in_sibling(focused_child)
        }
        Window::QuickFix(_) | Window::Tasks(_) | Window::Help(_) | Window::Directory(_, _, _) => {
            None
        }
    }
}

fn find_first_directory_by_focus(window: &Window) -> Option<(usize, usize, usize)> {
    match window {
        Window::Directory(parent, current, preview) => {
            Some((parent.buffer_id, current.buffer_id, preview.buffer_id))
        }
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
            SplitFocus::First => find_first_directory_by_focus(first),
            SplitFocus::Second => find_first_directory_by_focus(second),
        },
        Window::QuickFix(_) | Window::Tasks(_) | Window::Help(_) => None,
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use yeet_buffer::model::viewport::ViewPort;

    use crate::model::{qfix::QuickFix, App, Buffer, SplitFocus, Window};

    use super::{build_qfix_lines, focus_nearest_directory, open, remove_entry};

    fn make_qfix_with_entries(paths: Vec<PathBuf>) -> QuickFix {
        QuickFix {
            current_index: 0,
            entries: paths,
            ..Default::default()
        }
    }

    #[test]
    fn open_creates_horizontal_with_quickfix() {
        let mut app = App::default();
        let qfix = QuickFix::default();

        open(&mut app, None, &qfix);

        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(
            window,
            Window::Horizontal {
                focus: SplitFocus::Second,
                ..
            }
        ));

        match window {
            Window::Horizontal { first, second, .. } => {
                assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
                assert!(matches!(second.as_ref(), Window::QuickFix(_)));
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn open_with_entries_renders_formatted_lines() {
        let mut app = App::default();
        let qfix = make_qfix_with_entries(vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")]);

        open(&mut app, None, &qfix);

        let window = app.current_window().expect("test requires current tab");
        let qfix_vp = match window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::QuickFix(vp) => vp,
                _ => panic!("expected QuickFix"),
            },
            _ => panic!("expected Horizontal"),
        };

        let lines = match app.contents.buffers.get(&qfix_vp.buffer_id) {
            Some(Buffer::QuickFix(qb)) => &qb.buffer.lines,
            _ => panic!("expected Buffer::QuickFix"),
        };

        assert_eq!(lines.len(), 2);
        assert!(lines[0].content.to_stripped_string().contains("/tmp/a"));
        assert!(lines[1].content.to_stripped_string().contains("/tmp/b"));
    }

    #[test]
    fn open_with_empty_qfix_creates_empty_buffer() {
        let mut app = App::default();
        let qfix = QuickFix::default();

        open(&mut app, None, &qfix);

        let window = app.current_window().expect("test requires current tab");
        let qfix_vp = match window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::QuickFix(vp) => vp,
                _ => panic!("expected QuickFix"),
            },
            _ => panic!("expected Horizontal"),
        };

        match app.contents.buffers.get(&qfix_vp.buffer_id) {
            Some(Buffer::QuickFix(qb)) => assert!(qb.buffer.lines.is_empty()),
            _ => panic!("expected Buffer::QuickFix"),
        }
    }

    #[test]
    fn open_idempotent_switches_focus() {
        let mut app = App::default();
        let qfix = QuickFix::default();

        open(&mut app, None, &qfix);
        let window = app.current_window_mut().expect("test requires current tab");
        if let Window::Horizontal { focus, .. } = window {
            *focus = SplitFocus::First;
        }

        open(&mut app, None, &qfix);

        let window = app.current_window().expect("test requires current tab");
        match window {
            Window::Horizontal {
                first,
                second,
                focus,
            } => {
                assert_eq!(*focus, SplitFocus::Second);
                assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
                assert!(matches!(second.as_ref(), Window::QuickFix(_)));
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn build_qfix_lines_bold_current_index() {
        let qfix = QuickFix {
            current_index: 1,
            entries: vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c"),
            ],
            ..Default::default()
        };

        let lines = build_qfix_lines(&qfix, None);
        assert_eq!(lines.len(), 3);
        assert!(!lines[0].content.to_string().contains("\x1b[1m"));
        assert!(lines[1].content.to_string().contains("\x1b[1m"));
        assert!(!lines[2].content.to_string().contains("\x1b[1m"));
    }

    #[test]
    fn remove_entry_before_current_index_decrements() {
        let mut app = App::default();
        let mut qfix = make_qfix_with_entries(vec![
            PathBuf::from("/a"),
            PathBuf::from("/b"),
            PathBuf::from("/c"),
        ]);
        qfix.current_index = 2;
        open(&mut app, None, &qfix);

        remove_entry(&mut app, None, &mut qfix, 0);

        assert_eq!(qfix.current_index, 1);
        assert_eq!(qfix.entries.len(), 2);
    }

    #[test]
    fn remove_entry_at_current_index_clamps() {
        let mut app = App::default();
        let mut qfix = make_qfix_with_entries(vec![PathBuf::from("/a"), PathBuf::from("/b")]);
        qfix.current_index = 1;
        open(&mut app, None, &qfix);

        remove_entry(&mut app, None, &mut qfix, 1);

        assert_eq!(qfix.current_index, 0);
        assert_eq!(qfix.entries.len(), 1);
    }

    #[test]
    fn remove_entry_after_current_index_unchanged() {
        let mut app = App::default();
        let mut qfix = make_qfix_with_entries(vec![
            PathBuf::from("/a"),
            PathBuf::from("/b"),
            PathBuf::from("/c"),
        ]);
        qfix.current_index = 0;
        open(&mut app, None, &qfix);

        remove_entry(&mut app, None, &mut qfix, 2);

        assert_eq!(qfix.current_index, 0);
        assert_eq!(qfix.entries.len(), 2);
    }

    #[test]
    fn remove_last_entry_sets_index_to_zero() {
        let mut app = App::default();
        let mut qfix = make_qfix_with_entries(vec![PathBuf::from("/a")]);
        qfix.current_index = 0;
        open(&mut app, None, &qfix);

        remove_entry(&mut app, None, &mut qfix, 0);

        assert_eq!(qfix.current_index, 0);
        assert!(qfix.entries.is_empty());
    }

    #[test]
    fn remove_entry_cursor_past_end_clamps_to_last() {
        let mut app = App::default();
        let mut qfix = make_qfix_with_entries(vec![PathBuf::from("/a"), PathBuf::from("/b")]);
        qfix.current_index = 0;
        open(&mut app, None, &qfix);

        remove_entry(&mut app, None, &mut qfix, 5);

        assert_eq!(qfix.entries.len(), 1);
        assert_eq!(qfix.entries[0], PathBuf::from("/a"));
    }

    #[test]
    fn find_nearest_directory_simple_split() {
        let window = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort {
                    buffer_id: 10,
                    ..Default::default()
                },
                ViewPort::default(),
            )),
            second: Box::new(Window::QuickFix(ViewPort::default())),
            focus: SplitFocus::Second,
        };

        let result = super::find_nearest_directory_in_sibling(&window);
        assert!(result.is_some());
        let (_, current_id, _) = result.unwrap();
        assert_eq!(current_id, 10);
    }

    #[test]
    fn find_nearest_directory_nested_follows_focus() {
        let window = Window::Horizontal {
            first: Box::new(Window::Vertical {
                first: Box::new(Window::Directory(
                    ViewPort::default(),
                    ViewPort {
                        buffer_id: 10,
                        ..Default::default()
                    },
                    ViewPort::default(),
                )),
                second: Box::new(Window::Directory(
                    ViewPort::default(),
                    ViewPort {
                        buffer_id: 20,
                        ..Default::default()
                    },
                    ViewPort::default(),
                )),
                focus: SplitFocus::Second,
            }),
            second: Box::new(Window::QuickFix(ViewPort::default())),
            focus: SplitFocus::Second,
        };

        let result = super::find_nearest_directory_in_sibling(&window);
        assert!(result.is_some());
        let (_, current_id, _) = result.unwrap();
        assert_eq!(current_id, 20);
    }

    #[test]
    fn find_nearest_directory_no_directory_in_sibling() {
        let window = Window::Horizontal {
            first: Box::new(Window::Tasks(ViewPort::default())),
            second: Box::new(Window::QuickFix(ViewPort::default())),
            focus: SplitFocus::Second,
        };

        let result = super::find_nearest_directory_in_sibling(&window);
        assert!(result.is_none());
    }

    #[test]
    fn refresh_noop_without_quickfix_window() {
        let mut app = App::default();
        let qfix = QuickFix::default();
        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        super::refresh_quickfix_buffer_in_window(window, contents, &qfix, None);
    }

    #[test]
    fn refresh_updates_bold_on_index_change() {
        let mut app = App::default();
        let mut qfix = make_qfix_with_entries(vec![PathBuf::from("/a"), PathBuf::from("/b")]);
        open(&mut app, None, &qfix);

        qfix.current_index = 1;
        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        super::refresh_quickfix_buffer_in_window(window, contents, &qfix, None);

        let window = app.current_window().expect("test requires current tab");
        let qfix_vp = match window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::QuickFix(vp) => vp,
                _ => panic!("expected QuickFix"),
            },
            _ => panic!("expected Horizontal"),
        };

        let lines = match app.contents.buffers.get(&qfix_vp.buffer_id) {
            Some(Buffer::QuickFix(qb)) => &qb.buffer.lines,
            _ => panic!("expected Buffer::QuickFix"),
        };

        assert!(!lines[0].content.to_string().contains("\x1b[1m"));
        assert!(lines[1].content.to_string().contains("\x1b[1m"));
    }

    #[test]
    fn refresh_after_clear_shows_empty() {
        let mut app = App::default();
        let mut qfix = make_qfix_with_entries(vec![PathBuf::from("/a")]);
        open(&mut app, None, &qfix);

        qfix.entries.clear();
        qfix.current_index = 0;

        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");
        super::refresh_quickfix_buffer_in_window(window, contents, &qfix, None);

        let window = app.current_window().expect("test requires current tab");
        let qfix_vp = match window {
            Window::Horizontal { second, .. } => match second.as_ref() {
                Window::QuickFix(vp) => vp,
                _ => panic!("expected QuickFix"),
            },
            _ => panic!("expected Horizontal"),
        };

        match app.contents.buffers.get(&qfix_vp.buffer_id) {
            Some(Buffer::QuickFix(qb)) => assert!(qb.buffer.lines.is_empty()),
            _ => panic!("expected Buffer::QuickFix"),
        }
    }

    #[test]
    fn focus_nearest_directory_flips_to_sibling() {
        let mut window = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort::default(),
                ViewPort::default(),
                ViewPort::default(),
            )),
            second: Box::new(Window::QuickFix(ViewPort::default())),
            focus: SplitFocus::Second,
        };

        let result = focus_nearest_directory(&mut window);
        assert!(result);

        match &window {
            Window::Horizontal { focus, .. } => {
                assert_eq!(*focus, SplitFocus::First);
            }
            _ => panic!("expected Horizontal"),
        }
    }
}
