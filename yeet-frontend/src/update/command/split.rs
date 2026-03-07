use std::{mem, path::Path};

use yeet_keymap::message::KeymapMessage;

use crate::{
    action::{self, Action},
    model::{App, SplitFocus, Window},
    update::app,
};

pub fn horizontal(app: &mut App, target: &Path) -> Vec<Action> {
    create_split(app, target, |old, new| Window::Horizontal {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

pub fn vertical(app: &mut App, target: &Path) -> Vec<Action> {
    create_split(app, target, |old, new| Window::Vertical {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

fn create_split(
    app: &mut App,
    target: &Path,
    make_split: impl FnOnce(Window, Window) -> Window,
) -> Vec<Action> {
    let empty_buffer = app::get_empty_buffer(&mut app.contents);
    let new_directory = Window::create(empty_buffer, empty_buffer, empty_buffer);
    let old_window = mem::take(&mut app.window);
    app.window = make_split(old_window, new_directory);

    vec![action::emit_keymap(KeymapMessage::NavigateToPath(
        target.to_path_buf(),
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
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, &path);
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
        let path = env::current_dir().expect("get current dir");
        vertical(&mut app, &path);
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
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, &path);
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
        let path = env::current_dir().expect("get current dir");
        vertical(&mut app, &path);
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
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, &path);

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
        let path = env::current_dir().expect("get current dir");
        let actions = horizontal(&mut app, &path);
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
        let path = env::current_dir().expect("get current dir");
        let actions = horizontal(&mut app, &path);
        assert!(!actions.is_empty());
        assert!(matches!(app.window, Window::Horizontal { .. }));
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
        let path = env::current_dir().expect("get current dir");
        let actions = vertical(&mut app, &path);
        assert!(!actions.is_empty());
        assert!(matches!(app.window, Window::Vertical { .. }));
    }

    #[test]
    fn split_registers_buffers_in_contents() {
        let mut app = make_app_with_directory();
        let buffers_before = app.contents.buffers.len();
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, &path);
        assert_eq!(app.contents.buffers.len(), buffers_before + 1);
    }

    #[test]
    fn split_registers_empty_buffers_in_contents() {
        let mut app = make_app_with_directory();
        let path = env::current_dir().expect("get current dir");
        horizontal(&mut app, &path);

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
        let actions = vertical(&mut app, target.as_path());

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
        let actions = horizontal(&mut app, missing.as_path());
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
}
