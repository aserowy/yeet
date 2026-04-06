use std::{mem, path::Path};

use yeet_keymap::message::KeymapMessage;

use crate::{
    action::{self, Action},
    model::{App, SplitFocus, Window},
    update::app,
};

pub fn horizontal(app: &mut App, lua: Option<&yeet_lua::Lua>, target: &Path) -> Vec<Action> {
    create_split(app, lua, target, |old, new| Window::Horizontal {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

pub fn vertical(app: &mut App, lua: Option<&yeet_lua::Lua>, target: &Path) -> Vec<Action> {
    create_split(app, lua, target, |old, new| Window::Vertical {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

fn create_split(
    app: &mut App,
    lua: Option<&yeet_lua::Lua>,
    target: &Path,
    make_split: impl FnOnce(Window, Window) -> Window,
) -> Vec<Action> {
    let empty_buffer = app::get_empty_buffer(&mut app.contents);
    let mut new_directory = Window::create(empty_buffer, empty_buffer, empty_buffer);

    if let Some(lua) = lua {
        super::super::hook::on_window_create(lua, &mut new_directory, Some(target));
    }
    let window = match app.current_window_mut() {
        Ok(window) => window,
        Err(_) => return Vec::new(),
    };
    let focused_leaf = window.focused_window_mut();
    let old_window = mem::take(focused_leaf);
    *focused_leaf = make_split(old_window, new_directory);

    vec![action::emit_keymap(KeymapMessage::NavigateToPath(
        target.to_path_buf(),
    ))]
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::env;

    use yeet_buffer::model::viewport::ViewPort;

    use yeet_keymap::message::KeymapMessage;

    use crate::{
        event::Message,
        model::{App, Buffer, DirectoryBuffer, SplitFocus, Window},
    };

    use super::*;

    fn make_app_with_directory() -> App {
        let mut app = App::default();
        let path = env::current_dir().expect("get current dir");

        app.contents.buffers.insert(
            1,
            Buffer::Directory(DirectoryBuffer {
                path,
                ..Default::default()
            }),
        );
        let window = app.current_window_mut().expect("test requires current tab");
        if let Window::Directory(parent, current, preview) = window {
            parent.buffer_id = 1;
            current.buffer_id = 1;
            preview.buffer_id = 1;
        }

        app
    }

    fn make_directory_window(parent_id: usize, current_id: usize, preview_id: usize) -> Window {
        Window::Directory(
            ViewPort {
                buffer_id: parent_id,
                ..Default::default()
            },
            ViewPort {
                buffer_id: current_id,
                ..Default::default()
            },
            ViewPort {
                buffer_id: preview_id,
                ..Default::default()
            },
        )
    }

    #[test]
    fn horizontal_creates_horizontal_split() {
        let mut app = make_app_with_directory();
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, None, &path);
        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(
            window,
            Window::Horizontal {
                focus: SplitFocus::Second,
                ..
            }
        ));
    }

    #[test]
    fn vertical_creates_vertical_split() {
        let mut app = make_app_with_directory();
        let path = env::current_dir().expect("get current dir");
        vertical(&mut app, None, &path);
        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(
            window,
            Window::Vertical {
                focus: SplitFocus::Second,
                ..
            }
        ));
    }

    #[test]
    fn horizontal_first_child_is_original_directory() {
        let mut app = make_app_with_directory();
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, None, &path);
        let window = app.current_window().expect("test requires current tab");
        match window {
            Window::Horizontal { first, .. } => {
                assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn vertical_second_child_is_new_directory() {
        let mut app = make_app_with_directory();
        let path = env::current_dir().expect("get current dir");
        vertical(&mut app, None, &path);
        let window = app.current_window().expect("test requires current tab");
        match window {
            Window::Vertical { second, .. } => {
                assert!(matches!(second.as_ref(), Window::Directory(_, _, _)));
            }
            _ => panic!("expected Vertical"),
        }
    }

    #[test]
    fn split_allocates_fresh_buffer_ids() {
        let mut app = make_app_with_directory();
        let window = app.current_window().expect("test requires current tab");
        let original_ids = window.buffer_ids();
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, None, &path);

        let window = app.current_window().expect("test requires current tab");
        match window {
            Window::Horizontal { second, .. } => {
                let new_ids = second.buffer_ids();
                for id in &new_ids {
                    assert!(
                        !original_ids.contains(id),
                        "new pane should not share buffer IDs with original"
                    );
                }
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn split_returns_navigate_action() {
        let mut app = make_app_with_directory();
        let path = env::current_dir().expect("get current dir");
        let actions = horizontal(&mut app, None, &path);
        assert!(
            !actions.is_empty(),
            "split should return actions to load the new pane"
        );
        assert!(actions.iter().any(|action| match action {
            Action::EmitMessages(messages) => messages.iter().any(|message| {
                matches!(message, Message::Keymap(KeymapMessage::NavigateToPath(_)))
            }),
            _ => false,
        }));
    }

    #[test]
    fn split_noop_when_tasks_focused() {
        let mut app = App::default();
        let window = app.current_window_mut().expect("test requires current tab");
        *window = Window::Tasks(ViewPort::default());
        let path = env::current_dir().expect("get current dir");
        let actions = horizontal(&mut app, None, &path);
        assert!(!actions.is_empty());
        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Horizontal { .. }));
    }

    #[test]
    fn split_noop_when_tasks_focused_in_split() {
        let mut app = make_app_with_directory();
        let window = app.current_window_mut().expect("test requires current tab");
        let old_window = mem::take(window);
        *window = Window::Horizontal {
            first: Box::new(old_window),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::Second,
        };
        let path = env::current_dir().expect("get current dir");
        let actions = vertical(&mut app, None, &path);
        assert!(!actions.is_empty());
        let window = app.current_window().expect("test requires current tab");
        match window {
            Window::Horizontal { second, focus, .. } => {
                assert!(matches!(focus, SplitFocus::Second));
                assert!(matches!(second.as_ref(), Window::Vertical { .. }));
            }
            _ => panic!("expected root to remain horizontal"),
        }
    }

    #[test]
    fn split_registers_buffers_in_contents() {
        let mut app = make_app_with_directory();
        let buffers_before = app.contents.buffers.len();
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, None, &path);
        assert_eq!(app.contents.buffers.len(), buffers_before + 1);
    }

    #[test]
    fn split_registers_empty_buffers_in_contents() {
        let mut app = make_app_with_directory();
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, None, &path);

        let empty_count = app
            .contents
            .buffers
            .values()
            .filter(|buffer| matches!(buffer, Buffer::Empty))
            .count();

        assert!(empty_count >= 1);
    }

    #[test]
    fn split_with_path_navigates_to_target() {
        let mut app = make_app_with_directory();
        let target = env::temp_dir();
        let actions = vertical(&mut app, None, target.as_path());

        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(
            window,
            Window::Vertical {
                focus: SplitFocus::Second,
                ..
            }
        ));
        assert!(actions.iter().any(|action| match action {
            Action::EmitMessages(messages) => messages.iter().any(|message| {
                matches!(
                    message,
                    Message::Keymap(KeymapMessage::NavigateToPath(path)) if path == &target
                )
            }),
            _ => false,
        }));
    }

    #[test]
    fn split_with_nonexistent_path_returns_error() {
        let mut app = make_app_with_directory();
        let missing = std::path::PathBuf::from("/nonexistent/path/12345");
        let actions = horizontal(&mut app, None, missing.as_path());
        assert!(actions.iter().any(|action| match action {
            Action::EmitMessages(messages) => messages.iter().any(|message| {
                matches!(
                    message,
                    Message::Keymap(KeymapMessage::NavigateToPath(path)) if path == &missing
                )
            }),
            _ => false,
        }));
    }

    #[test]
    fn horizontal_split_targets_focused_leaf() {
        let mut app = make_app_with_directory();
        let window = app.current_window_mut().expect("test requires current tab");
        let left = make_directory_window(10, 11, 12);
        let inner_first = make_directory_window(20, 21, 22);
        let focused_leaf = make_directory_window(30, 31, 32);
        let focused_ids = HashSet::from([30, 31, 32]);

        *window = Window::Horizontal {
            first: Box::new(left),
            second: Box::new(Window::Vertical {
                first: Box::new(inner_first),
                second: Box::new(focused_leaf),
                focus: SplitFocus::Second,
            }),
            focus: SplitFocus::Second,
        };

        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, None, &path);

        let window = app.current_window().expect("test requires current tab");
        match window {
            Window::Horizontal {
                first,
                second,
                focus,
            } => {
                assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
                assert!(matches!(focus, SplitFocus::Second));
                match second.as_ref() {
                    Window::Vertical {
                        first,
                        second,
                        focus,
                    } => {
                        assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
                        assert!(matches!(focus, SplitFocus::Second));
                        match second.as_ref() {
                            Window::Horizontal { first, focus, .. } => {
                                assert!(matches!(focus, SplitFocus::Second));
                                assert_eq!(first.buffer_ids(), focused_ids);
                            }
                            _ => panic!("expected focused leaf to be horizontal split"),
                        }
                    }
                    _ => panic!("expected second child to remain vertical"),
                }
            }
            _ => panic!("expected root to remain horizontal"),
        }
    }

    #[test]
    fn vertical_split_targets_focused_leaf() {
        let mut app = make_app_with_directory();
        let window = app.current_window_mut().expect("test requires current tab");
        let left = make_directory_window(40, 41, 42);
        let inner_first = make_directory_window(50, 51, 52);
        let focused_leaf = make_directory_window(60, 61, 62);
        let focused_ids = HashSet::from([60, 61, 62]);

        *window = Window::Horizontal {
            first: Box::new(left),
            second: Box::new(Window::Vertical {
                first: Box::new(inner_first),
                second: Box::new(focused_leaf),
                focus: SplitFocus::Second,
            }),
            focus: SplitFocus::Second,
        };

        let path = env::current_dir().expect("get current dir");
        vertical(&mut app, None, &path);

        let window = app.current_window().expect("test requires current tab");
        match window {
            Window::Horizontal {
                first,
                second,
                focus,
            } => {
                assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
                assert!(matches!(focus, SplitFocus::Second));
                match second.as_ref() {
                    Window::Vertical {
                        first,
                        second,
                        focus,
                    } => {
                        assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
                        assert!(matches!(focus, SplitFocus::Second));
                        match second.as_ref() {
                            Window::Vertical { first, focus, .. } => {
                                assert!(matches!(focus, SplitFocus::Second));
                                assert_eq!(first.buffer_ids(), focused_ids);
                            }
                            _ => panic!("expected focused leaf to be vertical split"),
                        }
                    }
                    _ => panic!("expected second child to remain vertical"),
                }
            }
            _ => panic!("expected root to remain horizontal"),
        }
    }
}
