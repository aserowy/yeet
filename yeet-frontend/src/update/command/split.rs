use std::mem;

use yeet_buffer::model::viewport::{LineNumber, ViewPort};

use crate::{
    action::Action,
    model::{self, App, Buffer, SplitFocus, Window},
    update::app,
};

/// Creates a horizontal (top/bottom) split of the currently focused directory.
/// The new directory pane becomes the second child and receives focus.
pub fn horizontal(app: &mut App) -> Vec<Action> {
    create_split(app, |old, new| Window::Horizontal {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

/// Creates a vertical (left/right) split of the currently focused directory.
/// The new directory pane becomes the second child and receives focus.
pub fn vertical(app: &mut App) -> Vec<Action> {
    create_split(app, |old, new| Window::Vertical {
        first: Box::new(old),
        second: Box::new(new),
        focus: SplitFocus::Second,
    })
}

fn create_split(app: &mut App, make_split: impl FnOnce(Window, Window) -> Window) -> Vec<Action> {
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

    let selection_path = model::get_selected_path(current_buffer, &current_vp.cursor);
    let preview_path = selection_path.clone();
    let selection = selection_path.and_then(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
    });

    let parent_path = current_path.parent().map(|path| path.to_path_buf());

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
        .insert(current_id, Buffer::PathReference(current_path.clone()));

    if let Some(path) = &preview_path {
        app.contents
            .buffers
            .insert(preview_id, Buffer::PathReference(path.clone()));
    } else {
        app.contents.buffers.insert(preview_id, Buffer::Empty);
    }

    let new_directory = Window::Directory(
        ViewPort {
            buffer_id: parent_id,
            hide_cursor: true,
            show_border: true,
            ..Default::default()
        },
        ViewPort {
            buffer_id: current_id,
            line_number: LineNumber::Relative,
            line_number_width: 3,
            show_border: true,
            sign_column_width: 2,
            ..Default::default()
        },
        ViewPort {
            buffer_id: preview_id,
            hide_cursor: true,
            hide_cursor_line: true,
            ..Default::default()
        },
    );

    app.window.focused_viewport_mut().hide_cursor = true;

    let old_window = mem::take(&mut app.window);
    app.window = make_split(old_window, new_directory);

    let mut actions = Vec::new();
    if let Some(path) = parent_path {
        let selection = current_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string());
        actions.push(Action::Load(path, selection));
    }

    actions.push(Action::Load(current_path, selection));

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
        horizontal(&mut app);
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
        vertical(&mut app);
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
        horizontal(&mut app);
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
        vertical(&mut app);
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
        horizontal(&mut app);

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
        let actions = horizontal(&mut app);
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
        let actions = horizontal(&mut app);
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
        let actions = vertical(&mut app);
        assert!(actions.is_empty());
        assert!(matches!(app.window, Window::Horizontal { .. }));
    }

    #[test]
    fn split_registers_buffers_in_contents() {
        let mut app = make_app_with_directory();
        let buffers_before = app.contents.buffers.len();
        horizontal(&mut app);
        assert_eq!(app.contents.buffers.len(), buffers_before + 3);
    }
}
