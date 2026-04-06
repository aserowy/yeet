use yeet_keymap::message::FocusDirection;

use crate::{
    action::Action,
    model::{App, SplitFocus, Window},
};

pub fn change(app: &mut App, direction: &FocusDirection) -> Vec<Action> {
    if let Ok(window) = app.current_window_mut() {
        try_move(window, direction);
    }
    Vec::new()
}

fn try_move(window: &mut Window, direction: &FocusDirection) -> bool {
    match window {
        Window::Horizontal {
            first,
            second,
            focus,
        } => {
            let moved = match focus {
                SplitFocus::First => try_move(first, direction),
                SplitFocus::Second => try_move(second, direction),
            };
            if moved {
                return true;
            }
            match direction {
                FocusDirection::Down if *focus == SplitFocus::First => {
                    *focus = SplitFocus::Second;
                    enter_from(second, direction);
                    true
                }
                FocusDirection::Up if *focus == SplitFocus::Second => {
                    *focus = SplitFocus::First;
                    enter_from(first, direction);
                    true
                }
                _ => false,
            }
        }
        Window::Vertical {
            first,
            second,
            focus,
        } => {
            let moved = match focus {
                SplitFocus::First => try_move(first, direction),
                SplitFocus::Second => try_move(second, direction),
            };
            if moved {
                return true;
            }
            match direction {
                FocusDirection::Right if *focus == SplitFocus::First => {
                    *focus = SplitFocus::Second;
                    enter_from(second, direction);
                    true
                }
                FocusDirection::Left if *focus == SplitFocus::Second => {
                    *focus = SplitFocus::First;
                    enter_from(first, direction);
                    true
                }
                _ => false,
            }
        }
        Window::Directory(_, _, _) | Window::QuickFix(_) | Window::Tasks(_) | Window::Help(_) => {
            false
        }
    }
}

fn enter_from(window: &mut Window, direction: &FocusDirection) {
    match window {
        Window::Horizontal {
            first,
            second,
            focus,
        } => match direction {
            FocusDirection::Down | FocusDirection::Right => {
                *focus = SplitFocus::First;
                enter_from(first, direction);
            }
            FocusDirection::Up | FocusDirection::Left => {
                *focus = SplitFocus::Second;
                enter_from(second, direction);
            }
        },
        Window::Vertical {
            first,
            second,
            focus,
        } => match direction {
            FocusDirection::Right | FocusDirection::Down => {
                *focus = SplitFocus::First;
                enter_from(first, direction);
            }
            FocusDirection::Left | FocusDirection::Up => {
                *focus = SplitFocus::Second;
                enter_from(second, direction);
            }
        },
        Window::Directory(_, _, _) | Window::QuickFix(_) | Window::Tasks(_) | Window::Help(_) => {}
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use yeet_buffer::model::viewport::ViewPort;
    use yeet_keymap::message::FocusDirection;

    use crate::model::{App, Buffer, Contents, SplitFocus, TasksBuffer, Window};

    use super::change;

    fn current_window(app: &App) -> &Window {
        app.current_window().expect("test requires current tab")
    }

    fn make_horizontal_app() -> App {
        let mut buffers = HashMap::new();
        buffers.insert(10, Buffer::Empty);
        buffers.insert(11, Buffer::Empty);
        buffers.insert(12, Buffer::Empty);
        buffers.insert(20, Buffer::Tasks(TasksBuffer::default()));

        let mut tabs = HashMap::new();
        tabs.insert(
            1,
            Window::Horizontal {
                first: Box::new(Window::Directory(
                    ViewPort {
                        buffer_id: 10,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 11,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 12,
                        ..Default::default()
                    },
                )),
                second: Box::new(Window::Tasks(ViewPort {
                    buffer_id: 20,
                    ..Default::default()
                })),
                focus: SplitFocus::First,
            },
        );

        App {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 20,
            },
            tabs,
            current_tab_id: 1,
        }
    }

    #[test]
    fn down_moves_focus_to_second() {
        let mut app = make_horizontal_app();
        change(&mut app, &FocusDirection::Down);

        match current_window(&app) {
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

        match current_window(&app) {
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
        match current_window(&app) {
            Window::Horizontal { focus, .. } => assert_eq!(*focus, SplitFocus::First),
            _ => panic!("expected Horizontal"),
        }

        change(&mut app, &FocusDirection::Right);
        match current_window(&app) {
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

    fn make_vertical_app() -> App {
        let mut buffers = HashMap::new();
        buffers.insert(10, Buffer::Empty);
        buffers.insert(11, Buffer::Empty);
        buffers.insert(12, Buffer::Empty);
        buffers.insert(20, Buffer::Empty);
        buffers.insert(21, Buffer::Empty);
        buffers.insert(22, Buffer::Empty);

        let mut tabs = HashMap::new();
        tabs.insert(
            1,
            Window::Vertical {
                first: Box::new(Window::Directory(
                    ViewPort {
                        buffer_id: 10,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 11,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 12,
                        ..Default::default()
                    },
                )),
                second: Box::new(Window::Directory(
                    ViewPort {
                        buffer_id: 20,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 21,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 22,
                        ..Default::default()
                    },
                )),
                focus: SplitFocus::First,
            },
        );

        App {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 22,
            },
            tabs,
            current_tab_id: 1,
        }
    }

    #[test]
    fn right_moves_focus_on_vertical() {
        let mut app = make_vertical_app();
        change(&mut app, &FocusDirection::Right);
        match current_window(&app) {
            Window::Vertical { focus, .. } => assert_eq!(*focus, SplitFocus::Second),
            _ => panic!("expected Vertical"),
        }
    }

    #[test]
    fn left_moves_focus_on_vertical() {
        let mut app = make_vertical_app();
        // First move right to second, then left back to first
        change(&mut app, &FocusDirection::Right);
        change(&mut app, &FocusDirection::Left);
        match current_window(&app) {
            Window::Vertical { focus, .. } => assert_eq!(*focus, SplitFocus::First),
            _ => panic!("expected Vertical"),
        }
    }

    #[test]
    fn up_down_noop_on_vertical() {
        let mut app = make_vertical_app();
        change(&mut app, &FocusDirection::Up);
        match current_window(&app) {
            Window::Vertical { focus, .. } => assert_eq!(*focus, SplitFocus::First),
            _ => panic!("expected Vertical"),
        }
        change(&mut app, &FocusDirection::Down);
        match current_window(&app) {
            Window::Vertical { focus, .. } => assert_eq!(*focus, SplitFocus::First),
            _ => panic!("expected Vertical"),
        }
    }

    /// ```text
    /// Horizontal {
    ///     first: Vertical { first: Dir_A, second: Dir_B, focus: First },
    ///     second: Tasks,
    ///     focus: First,
    /// }
    /// ```
    /// Layout:
    /// ```text
    /// +--------+--------+
    /// | Dir_A  | Dir_B  |
    /// +--------+--------+
    /// |     Tasks        |
    /// +------------------+
    /// ```
    fn make_vertical_inside_horizontal_app() -> App {
        let mut buffers = HashMap::new();
        // Dir_A viewports: 10, 11, 12
        buffers.insert(10, Buffer::Empty);
        buffers.insert(11, Buffer::Empty);
        buffers.insert(12, Buffer::Empty);
        // Dir_B viewports: 20, 21, 22
        buffers.insert(20, Buffer::Empty);
        buffers.insert(21, Buffer::Empty);
        buffers.insert(22, Buffer::Empty);
        // Tasks viewport: 30
        buffers.insert(30, Buffer::Tasks(TasksBuffer::default()));

        let mut tabs = HashMap::new();
        tabs.insert(
            1,
            Window::Horizontal {
                first: Box::new(Window::Vertical {
                    first: Box::new(Window::Directory(
                        ViewPort {
                            buffer_id: 10,
                            ..Default::default()
                        },
                        ViewPort {
                            buffer_id: 11,
                            ..Default::default()
                        },
                        ViewPort {
                            buffer_id: 12,
                            ..Default::default()
                        },
                    )),
                    second: Box::new(Window::Directory(
                        ViewPort {
                            buffer_id: 20,
                            ..Default::default()
                        },
                        ViewPort {
                            buffer_id: 21,
                            ..Default::default()
                        },
                        ViewPort {
                            buffer_id: 22,
                            ..Default::default()
                        },
                    )),
                    focus: SplitFocus::First,
                }),
                second: Box::new(Window::Tasks(ViewPort {
                    buffer_id: 30,
                    ..Default::default()
                })),
                focus: SplitFocus::First,
            },
        );

        App {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 30,
            },
            tabs,
            current_tab_id: 1,
        }
    }

    #[test]
    fn nested_vert_in_horiz_right_moves_within_vertical() {
        let mut app = make_vertical_inside_horizontal_app();
        // Focus on Dir_A, press Right → Dir_B
        change(&mut app, &FocusDirection::Right);

        match current_window(&app) {
            Window::Horizontal { first, focus, .. } => {
                assert_eq!(*focus, SplitFocus::First, "outer focus stays on first");
                match first.as_ref() {
                    Window::Vertical { focus, .. } => {
                        assert_eq!(*focus, SplitFocus::Second, "inner focus moves to Dir_B");
                    }
                    _ => panic!("expected Vertical"),
                }
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn nested_vert_in_horiz_down_crosses_to_tasks() {
        let mut app = make_vertical_inside_horizontal_app();
        // Focus on Dir_A, press Down → Tasks
        change(&mut app, &FocusDirection::Down);

        match current_window(&app) {
            Window::Horizontal { focus, .. } => {
                assert_eq!(*focus, SplitFocus::Second, "outer focus moves to Tasks");
            }
            _ => panic!("expected Horizontal"),
        }
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 30);
    }

    #[test]
    fn nested_vert_in_horiz_down_from_dir_b_crosses_to_tasks() {
        let mut app = make_vertical_inside_horizontal_app();
        // Move to Dir_B first
        change(&mut app, &FocusDirection::Right);
        // Now press Down → Tasks
        change(&mut app, &FocusDirection::Down);

        match current_window(&app) {
            Window::Horizontal { focus, .. } => {
                assert_eq!(*focus, SplitFocus::Second, "outer focus moves to Tasks");
            }
            _ => panic!("expected Horizontal"),
        }
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 30);
    }

    #[test]
    fn nested_vert_in_horiz_up_from_tasks_enters_vertical_first() {
        let mut app = make_vertical_inside_horizontal_app();
        // Move to Tasks
        change(&mut app, &FocusDirection::Down);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 30);

        // Press Up → enters Vertical from Up → focuses second (right/bottom)
        // because enter_from with Up on Vertical picks SplitFocus::Second
        change(&mut app, &FocusDirection::Up);

        match current_window(&app) {
            Window::Horizontal { first, focus, .. } => {
                assert_eq!(*focus, SplitFocus::First);
                match first.as_ref() {
                    Window::Vertical { focus, .. } => {
                        assert_eq!(
                            *focus,
                            SplitFocus::Second,
                            "entering Vertical from Up should focus second (right)"
                        );
                    }
                    _ => panic!("expected Vertical"),
                }
            }
            _ => panic!("expected Horizontal"),
        }
    }

    #[test]
    fn nested_vert_in_horiz_left_noop_on_dir_a() {
        let mut app = make_vertical_inside_horizontal_app();
        // Focus on Dir_A, press Left → no-op (already leftmost)
        change(&mut app, &FocusDirection::Left);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 11);
    }

    #[test]
    fn nested_vert_in_horiz_up_noop_on_dir_a() {
        let mut app = make_vertical_inside_horizontal_app();
        // Focus on Dir_A, press Up → no-op (already topmost)
        change(&mut app, &FocusDirection::Up);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 11);
    }

    /// ```text
    /// Vertical {
    ///     first: Horizontal { first: Dir_A, second: Tasks, focus: First },
    ///     second: Dir_B,
    ///     focus: First,
    /// }
    /// ```
    /// Layout:
    /// ```text
    /// +--------+--------+
    /// | Dir_A  |        |
    /// +--------+ Dir_B  |
    /// | Tasks  |        |
    /// +--------+--------+
    /// ```
    fn make_horizontal_inside_vertical_app() -> App {
        let mut buffers = HashMap::new();
        // Dir_A viewports: 10, 11, 12
        buffers.insert(10, Buffer::Empty);
        buffers.insert(11, Buffer::Empty);
        buffers.insert(12, Buffer::Empty);
        // Tasks viewport: 20
        buffers.insert(20, Buffer::Tasks(TasksBuffer::default()));
        // Dir_B viewports: 30, 31, 32
        buffers.insert(30, Buffer::Empty);
        buffers.insert(31, Buffer::Empty);
        buffers.insert(32, Buffer::Empty);

        let mut tabs = HashMap::new();
        tabs.insert(
            1,
            Window::Vertical {
                first: Box::new(Window::Horizontal {
                    first: Box::new(Window::Directory(
                        ViewPort {
                            buffer_id: 10,
                            ..Default::default()
                        },
                        ViewPort {
                            buffer_id: 11,
                            ..Default::default()
                        },
                        ViewPort {
                            buffer_id: 12,
                            ..Default::default()
                        },
                    )),
                    second: Box::new(Window::Tasks(ViewPort {
                        buffer_id: 20,
                        ..Default::default()
                    })),
                    focus: SplitFocus::First,
                }),
                second: Box::new(Window::Directory(
                    ViewPort {
                        buffer_id: 30,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 31,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 32,
                        ..Default::default()
                    },
                )),
                focus: SplitFocus::First,
            },
        );

        App {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 32,
            },
            tabs,
            current_tab_id: 1,
        }
    }

    #[test]
    fn nested_horiz_in_vert_right_crosses_to_dir_b() {
        let mut app = make_horizontal_inside_vertical_app();
        // Focus on Dir_A, press Right → Dir_B (crosses Vertical boundary)
        change(&mut app, &FocusDirection::Right);

        match current_window(&app) {
            Window::Vertical { focus, .. } => {
                assert_eq!(*focus, SplitFocus::Second, "outer focus moves to Dir_B");
            }
            _ => panic!("expected Vertical"),
        }
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 31);
    }

    #[test]
    fn nested_horiz_in_vert_down_moves_within_horizontal() {
        let mut app = make_horizontal_inside_vertical_app();
        // Focus on Dir_A, press Down → Tasks (within the Horizontal)
        change(&mut app, &FocusDirection::Down);

        match current_window(&app) {
            Window::Vertical { first, focus, .. } => {
                assert_eq!(*focus, SplitFocus::First, "outer focus stays on first");
                match first.as_ref() {
                    Window::Horizontal { focus, .. } => {
                        assert_eq!(*focus, SplitFocus::Second, "inner focus moves to Tasks");
                    }
                    _ => panic!("expected Horizontal"),
                }
            }
            _ => panic!("expected Vertical"),
        }
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 20);
    }

    #[test]
    fn nested_horiz_in_vert_left_from_dir_b_enters_horizontal_from_right() {
        let mut app = make_horizontal_inside_vertical_app();
        // Move to Dir_B
        change(&mut app, &FocusDirection::Right);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 31);

        // Press Left → enters Horizontal from right → focuses second (Tasks)
        change(&mut app, &FocusDirection::Left);

        match current_window(&app) {
            Window::Vertical { first, focus, .. } => {
                assert_eq!(*focus, SplitFocus::First);
                match first.as_ref() {
                    Window::Horizontal { focus, .. } => {
                        assert_eq!(
                            *focus,
                            SplitFocus::Second,
                            "entering Horizontal from Left should focus second (bottom)"
                        );
                    }
                    _ => panic!("expected Horizontal"),
                }
            }
            _ => panic!("expected Vertical"),
        }
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 20);
    }

    #[test]
    fn nested_horiz_in_vert_right_from_tasks_crosses_to_dir_b() {
        let mut app = make_horizontal_inside_vertical_app();
        // Move to Tasks
        change(&mut app, &FocusDirection::Down);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 20);

        // Press Right → crosses Vertical boundary to Dir_B
        change(&mut app, &FocusDirection::Right);

        match current_window(&app) {
            Window::Vertical { focus, .. } => {
                assert_eq!(*focus, SplitFocus::Second);
            }
            _ => panic!("expected Vertical"),
        }
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 31);
    }

    /// Deep nesting: 3 levels
    /// ```text
    /// Horizontal {
    ///     first: Vertical {
    ///         first: Dir_A,
    ///         second: Horizontal {
    ///             first: Dir_B,
    ///             second: Dir_C,
    ///             focus: First,
    ///         },
    ///         focus: First,
    ///     },
    ///     second: Tasks,
    ///     focus: First,
    /// }
    /// ```
    fn make_deep_nested_app() -> App {
        let mut buffers = HashMap::new();
        // Dir_A: 10, 11, 12
        buffers.insert(10, Buffer::Empty);
        buffers.insert(11, Buffer::Empty);
        buffers.insert(12, Buffer::Empty);
        // Dir_B: 20, 21, 22
        buffers.insert(20, Buffer::Empty);
        buffers.insert(21, Buffer::Empty);
        buffers.insert(22, Buffer::Empty);
        // Dir_C: 30, 31, 32
        buffers.insert(30, Buffer::Empty);
        buffers.insert(31, Buffer::Empty);
        buffers.insert(32, Buffer::Empty);
        // Tasks: 40
        buffers.insert(40, Buffer::Tasks(TasksBuffer::default()));

        let mut tabs = HashMap::new();
        tabs.insert(
            1,
            Window::Horizontal {
                first: Box::new(Window::Vertical {
                    first: Box::new(Window::Directory(
                        ViewPort {
                            buffer_id: 10,
                            ..Default::default()
                        },
                        ViewPort {
                            buffer_id: 11,
                            ..Default::default()
                        },
                        ViewPort {
                            buffer_id: 12,
                            ..Default::default()
                        },
                    )),
                    second: Box::new(Window::Horizontal {
                        first: Box::new(Window::Directory(
                            ViewPort {
                                buffer_id: 20,
                                ..Default::default()
                            },
                            ViewPort {
                                buffer_id: 21,
                                ..Default::default()
                            },
                            ViewPort {
                                buffer_id: 22,
                                ..Default::default()
                            },
                        )),
                        second: Box::new(Window::Directory(
                            ViewPort {
                                buffer_id: 30,
                                ..Default::default()
                            },
                            ViewPort {
                                buffer_id: 31,
                                ..Default::default()
                            },
                            ViewPort {
                                buffer_id: 32,
                                ..Default::default()
                            },
                        )),
                        focus: SplitFocus::First,
                    }),
                    focus: SplitFocus::First,
                }),
                second: Box::new(Window::Tasks(ViewPort {
                    buffer_id: 40,
                    ..Default::default()
                })),
                focus: SplitFocus::First,
            },
        );

        App {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 40,
            },
            tabs,
            current_tab_id: 1,
        }
    }

    #[test]
    fn deep_nested_right_from_dir_a_to_dir_b() {
        let mut app = make_deep_nested_app();
        // Dir_A, press Right → enters inner Vertical second → inner Horizontal → Dir_B (first/top)
        change(&mut app, &FocusDirection::Right);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 21);
    }

    #[test]
    fn deep_nested_down_from_dir_a_to_tasks() {
        let mut app = make_deep_nested_app();
        // Dir_A, press Down → crosses outer Horizontal to Tasks
        change(&mut app, &FocusDirection::Down);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 40);
    }

    #[test]
    fn deep_nested_down_from_dir_b_to_dir_c() {
        let mut app = make_deep_nested_app();
        // Navigate to Dir_B
        change(&mut app, &FocusDirection::Right);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 21);

        // Dir_B, press Down → Dir_C (within inner Horizontal)
        change(&mut app, &FocusDirection::Down);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 31);
    }

    #[test]
    fn deep_nested_left_from_dir_b_to_dir_a() {
        let mut app = make_deep_nested_app();
        // Navigate to Dir_B
        change(&mut app, &FocusDirection::Right);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 21);

        // Dir_B, press Left → Dir_A (Vertical second → first)
        change(&mut app, &FocusDirection::Left);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 11);
    }

    #[test]
    fn deep_nested_left_from_dir_c_to_dir_a() {
        let mut app = make_deep_nested_app();
        // Navigate to Dir_B then Dir_C
        change(&mut app, &FocusDirection::Right);
        change(&mut app, &FocusDirection::Down);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 31);

        // Dir_C, press Left → Dir_A (inner Horizontal can't handle Left, bubbles to Vertical → first)
        change(&mut app, &FocusDirection::Left);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 11);
    }

    #[test]
    fn deep_nested_down_from_dir_c_to_tasks() {
        let mut app = make_deep_nested_app();
        // Navigate to Dir_C
        change(&mut app, &FocusDirection::Right);
        change(&mut app, &FocusDirection::Down);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 31);

        // Dir_C, press Down → inner Horizontal can't handle (already second),
        // bubbles to Vertical (not relevant for Down), bubbles to outer Horizontal → Tasks
        change(&mut app, &FocusDirection::Down);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 40);
    }

    #[test]
    fn deep_nested_right_noop_from_dir_b() {
        let mut app = make_deep_nested_app();
        // Navigate to Dir_B
        change(&mut app, &FocusDirection::Right);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 21);

        // Dir_B, press Right → no Vertical ancestor to the right → no-op
        change(&mut app, &FocusDirection::Right);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 21);
    }

    #[test]
    fn deep_nested_up_from_tasks_enters_vertical_first() {
        let mut app = make_deep_nested_app();
        // Navigate to Tasks
        change(&mut app, &FocusDirection::Down);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 40);

        // Tasks, press Up → outer Horizontal second→first → enters Vertical from Up
        // enter_from with Up on Vertical → SplitFocus::Second → enters inner Horizontal from Up
        // enter_from with Up on Horizontal → SplitFocus::Second → Dir_C (current vp = 31)
        change(&mut app, &FocusDirection::Up);
        assert_eq!(current_window(&app).focused_viewport().buffer_id, 31);
    }
}
