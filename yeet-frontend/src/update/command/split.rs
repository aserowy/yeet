use std::mem;

use yeet_keymap::message::KeymapMessage;

use crate::{
    action::{self, Action},
    event::Message,
    model::{App, SplitFocus, Window},
    update::app,
};

pub fn horizontal(app: &mut App, target: Option<std::path::PathBuf>) -> Vec<Action> {
    create_split(app, target, |old, new| Window::Horizontal {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

pub fn vertical(app: &mut App, target: Option<std::path::PathBuf>) -> Vec<Action> {
    create_split(app, target, |old, new| Window::Vertical {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

fn create_split(
    app: &mut App,
    target: Option<std::path::PathBuf>,
    make_split: impl FnOnce(Window, Window) -> Window,
) -> Vec<Action> {
    let (_, current_id, _) = match app::get_focused_directory_buffer_ids(&app.window) {
        Some(ids) => ids,
        None => return Vec::new(),
    };

    let current_path = match app::get_buffer_path(app, current_id) {
        Some(path) => path.to_path_buf(),
        None => return Vec::new(),
    };

    let navigate_path = match &target {
        Some(path) => {
            let resolved = if path.is_relative() {
                current_path.join(path)
            } else {
                path.clone()
            };

            if !resolved.exists() {
                return vec![Action::EmitMessages(vec![Message::Error(format!(
                    "Path does not exist: {}",
                    resolved.display()
                ))])];
            }

            resolved
        }
        None => current_path.clone(),
    };

    let empty_buffer = app::get_empty_buffer(&mut app.contents);
    let new_directory = Window::create(empty_buffer, empty_buffer, empty_buffer);
    let old_window = mem::take(&mut app.window);
    app.window = make_split(old_window, new_directory);

    vec![action::emit_keymap(KeymapMessage::NavigateToPath(
        navigate_path,
    ))]
}

#[cfg(test)]
mod test {
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
        if let Window::Directory(parent, current, preview) = &mut app.window {
            parent.buffer_id = 1;
            current.buffer_id = 1;
            preview.buffer_id = 1;
        }

        app
    }

    #[test]
    fn horizontal_creates_horizontal_split() {
        let mut app = make_app_with_directory();
        horizontal(&mut app, None);
        assert!(matches!(
            app.window,
            Window::Horizontal {
                focus: SplitFocus::Second,
                ..
            }
        ));
    }

    #[test]
    fn vertical_creates_vertical_split() {
        let mut app = make_app_with_directory();
        vertical(&mut app, None);
        assert!(matches!(
            app.window,
            Window::Vertical {
                focus: SplitFocus::Second,
                ..
            }
        ));
    }

    #[test]
    fn horizontal_first_child_is_original_directory() {
        let mut app = make_app_with_directory();
        horizontal(&mut app, None);
        match &app.window {
            Window::Horizontal { first, .. } => {
                assert!(matches!(first.as_ref(), Window::Directory(_, _, _)));
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn vertical_second_child_is_new_directory() {
        let mut app = make_app_with_directory();
        vertical(&mut app, None);
        match &app.window {
            Window::Vertical { second, .. } => {
                assert!(matches!(second.as_ref(), Window::Directory(_, _, _)));
            }
            _ => panic!("expected Vertical"),
        }
    }

    #[test]
    fn split_allocates_fresh_buffer_ids() {
        let mut app = make_app_with_directory();
        let original_ids = app.window.buffer_ids();
        horizontal(&mut app, None);

        match &app.window {
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
        let actions = horizontal(&mut app, None);
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
        let mut app = App {
            window: Window::Tasks(ViewPort::default()),
            ..Default::default()
        };
        let actions = horizontal(&mut app, None);
        assert!(actions.is_empty());
        assert!(matches!(app.window, Window::Tasks(_)));
    }

    #[test]
    fn split_noop_when_tasks_focused_in_split() {
        let mut app = make_app_with_directory();
        let old_window = mem::take(&mut app.window);
        app.window = Window::Horizontal {
            first: Box::new(old_window),
            second: Box::new(Window::Tasks(ViewPort::default())),
            focus: SplitFocus::Second,
        };
        let actions = vertical(&mut app, None);
        assert!(actions.is_empty());
        assert!(matches!(app.window, Window::Horizontal { .. }));
    }

    #[test]
    fn split_registers_buffers_in_contents() {
        let mut app = make_app_with_directory();
        let buffers_before = app.contents.buffers.len();
        horizontal(&mut app, None);
        assert_eq!(app.contents.buffers.len(), buffers_before + 3);
    }

    #[test]
    fn split_registers_empty_buffers_in_contents() {
        let mut app = make_app_with_directory();
        horizontal(&mut app, None);

        let empty_count = app
            .contents
            .buffers
            .values()
            .filter(|buffer| matches!(buffer, Buffer::Empty))
            .count();

        assert!(empty_count >= 3);
    }

    #[test]
    fn split_with_path_navigates_to_target() {
        let mut app = make_app_with_directory();
        let target = env::temp_dir();
        let actions = vertical(&mut app, Some(target.clone()));

        assert!(matches!(
            app.window,
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
        let actions = horizontal(&mut app, Some(missing));
        assert!(matches!(app.window, Window::Directory(_, _, _)));
        assert!(actions
            .iter()
            .any(|action| matches!(action, Action::EmitMessages(_))));
    }
}
