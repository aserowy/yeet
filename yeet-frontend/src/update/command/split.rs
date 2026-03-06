use std::mem;

use crate::{
    action::Action,
    event::Message,
    model::{self, App, Buffer, SplitFocus, Window},
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
    let (_, current_vp, _) = match app::get_focused_directory_viewports(&app.window) {
        Some(vps) => vps,
        None => return Vec::new(),
    };

    let current_buffer = match app.contents.buffers.get(&current_vp.buffer_id) {
        Some(Buffer::Directory(buffer)) => buffer,
        _ => return Vec::new(),
    };

    let current_path = current_buffer.path.clone();
    if current_path.as_os_str().is_empty() {
        return Vec::new();
    }

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

    let selection_path = match target {
        Some(_) => None,
        None => model::get_selected_path(current_buffer, &current_vp.cursor),
    };
    let preview_path = selection_path.clone();
    let selection = selection_path.and_then(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
    });

    let parent_path = navigate_path.parent().map(|path| path.to_path_buf());

    let parent_id = app::get_next_buffer_id(&mut app.contents);
    let current_id = app::get_next_buffer_id(&mut app.contents);
    let preview_id = app::get_next_buffer_id(&mut app.contents);

    if let Some(path) = &parent_path {
        app.contents
            .buffers
            .insert(parent_id, Buffer::PathReference(path.clone()));
    } else {
        app.contents.buffers.insert(parent_id, Buffer::Empty);
    }

    app.contents
        .buffers
        .insert(current_id, Buffer::PathReference(navigate_path.clone()));

    if let Some(path) = &preview_path {
        app.contents
            .buffers
            .insert(preview_id, Buffer::PathReference(path.clone()));
    } else {
        app.contents.buffers.insert(preview_id, Buffer::Empty);
    }

    let new_directory = Window::create(parent_id, current_id, preview_id);
    app.window.focused_viewport_mut().hide_cursor = true;

    let old_window = mem::take(&mut app.window);
    app.window = make_split(old_window, new_directory);

    let mut actions = Vec::new();
    if let Some(path) = parent_path {
        let selection = navigate_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string());
        actions.push(Action::Load(path, selection));
    }

    actions.push(Action::Load(navigate_path, selection));

    if let Some(path) = preview_path {
        actions.push(Action::Load(path, None));
    }

    actions
}

#[cfg(test)]
mod test {
    use std::env;

    use yeet_buffer::model::viewport::ViewPort;

    use crate::model::{App, Buffer, DirectoryBuffer, SplitFocus, Window};

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
    fn split_returns_load_actions() {
        let mut app = make_app_with_directory();
        let actions = horizontal(&mut app, None);
        assert!(
            !actions.is_empty(),
            "split should return actions to load the new pane"
        );
        assert!(actions
            .iter()
            .any(|action| matches!(action, Action::Load(_, _))));
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
    fn split_with_path_sets_path_references() {
        let mut app = make_app_with_directory();
        let target = env::temp_dir();
        vertical(&mut app, Some(target.clone()));

        let buffer_paths: Vec<_> = app
            .contents
            .buffers
            .values()
            .filter_map(|buffer| match buffer {
                Buffer::PathReference(path) => Some(path.clone()),
                _ => None,
            })
            .collect();

        assert!(buffer_paths.iter().any(|path| path == &target));
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
