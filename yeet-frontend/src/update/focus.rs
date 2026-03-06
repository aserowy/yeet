use yeet_keymap::message::FocusDirection;

use crate::{
    action::Action,
    model::{App, SplitFocus, Window},
};

pub fn change(app: &mut App, direction: &FocusDirection) -> Vec<Action> {
    let (first, second, focus) = match &mut app.window {
        Window::Horizontal {
            first,
            second,
            focus,
        } => (first, second, focus),
        Window::Vertical { .. } => todo!(),
        _ => return Vec::new(),
    };

    let new_focus = match direction {
        FocusDirection::Down => SplitFocus::Second,
        FocusDirection::Up => SplitFocus::First,
        FocusDirection::Left | FocusDirection::Right => return Vec::new(),
    };

    if *focus == new_focus {
        return Vec::new();
    }

    // Hide cursor on the old focused leaf, show on the new one.
    match focus {
        SplitFocus::First => {
            first.focused_viewport_mut().hide_cursor = true;
            second.focused_viewport_mut().hide_cursor = false;
        }
        SplitFocus::Second => {
            second.focused_viewport_mut().hide_cursor = true;
            first.focused_viewport_mut().hide_cursor = false;
        }
    }

    *focus = new_focus;

    Vec::new()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use yeet_buffer::model::viewport::ViewPort;
    use yeet_keymap::message::FocusDirection;

    use crate::model::{App, Buffer, Contents, SplitFocus, TasksBuffer, Window};

    use super::change;

    fn make_horizontal_app() -> App {
        let mut buffers = HashMap::new();
        buffers.insert(10, Buffer::Empty);
        buffers.insert(11, Buffer::Empty);
        buffers.insert(12, Buffer::Empty);
        buffers.insert(20, Buffer::Tasks(TasksBuffer::default()));

        App {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 20,
            },
            window: Window::Horizontal {
                first: Box::new(Window::Directory(
                    ViewPort {
                        buffer_id: 10,
                        hide_cursor: true,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 11,
                        hide_cursor: false,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 12,
                        hide_cursor: true,
                        ..Default::default()
                    },
                )),
                second: Box::new(Window::Tasks(ViewPort {
                    buffer_id: 20,
                    hide_cursor: true,
                    ..Default::default()
                })),
                focus: SplitFocus::First,
            },
        }
    }

    #[test]
    fn down_moves_focus_to_second() {
        let mut app = make_horizontal_app();
        change(&mut app, &FocusDirection::Down);

        match &app.window {
            Window::Horizontal { focus, .. } => {
                assert_eq!(*focus, SplitFocus::Second);
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn up_when_already_first_is_noop() {
        let mut app = make_horizontal_app();
        change(&mut app, &FocusDirection::Up);

        match &app.window {
            Window::Horizontal { focus, .. } => {
                assert_eq!(*focus, SplitFocus::First);
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn left_right_are_noops_on_horizontal() {
        let mut app = make_horizontal_app();
        change(&mut app, &FocusDirection::Left);
        match &app.window {
            Window::Horizontal { focus, .. } => assert_eq!(*focus, SplitFocus::First),
            _ => panic!("expected Horizontal"),
        }

        change(&mut app, &FocusDirection::Right);
        match &app.window {
            Window::Horizontal { focus, .. } => assert_eq!(*focus, SplitFocus::First),
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn all_directions_noop_on_directory_root() {
        let mut app = App::default();
        for direction in &[
            FocusDirection::Up,
            FocusDirection::Down,
            FocusDirection::Left,
            FocusDirection::Right,
        ] {
            let actions = change(&mut app, direction);
            assert!(actions.is_empty());
        }
    }

    #[test]
    fn cursor_visibility_toggles_on_focus_change() {
        let mut app = make_horizontal_app();

        // Initially: first focused, directory current vp has hide_cursor=false, tasks has hide_cursor=true
        assert!(!app.window.focused_viewport().hide_cursor);

        change(&mut app, &FocusDirection::Down);

        // After: second focused (tasks), tasks vp should have hide_cursor=false
        match &app.window {
            Window::Horizontal {
                first,
                second,
                focus,
            } => {
                assert_eq!(*focus, SplitFocus::Second);
                // Old focused leaf (directory current vp) should have cursor hidden
                assert!(first.focused_viewport().hide_cursor);
                // New focused leaf (tasks vp) should have cursor shown
                assert!(!second.focused_viewport().hide_cursor);
            }
            _ => panic!("expected Horizontal"),
        }

        // Change back
        change(&mut app, &FocusDirection::Up);

        match &app.window {
            Window::Horizontal {
                first,
                second,
                focus,
            } => {
                assert_eq!(*focus, SplitFocus::First);
                assert!(!first.focused_viewport().hide_cursor);
                assert!(second.focused_viewport().hide_cursor);
            }
            _ => panic!("expected Horizontal"),
        }
    }
}
