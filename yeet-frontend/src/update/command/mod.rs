use std::{mem, path::Path};

use yeet_buffer::{message::BufferMessage, model::Mode};
use yeet_keymap::message::{KeymapMessage, QuitMode};

use crate::{
    action::{self, Action},
    event::Message,
    model::{App, Buffer, Contents, SplitFocus, State, Window},
    task::Task,
    update::{app, tab},
};

mod file;
mod print;
mod qfix;
mod split;
pub mod task;

#[tracing::instrument(skip_all)]
pub fn execute(app: &mut App, state: &mut State, cmd: &str) -> Vec<Action> {
    let cmd_with_args = match cmd.split_once(' ') {
        Some(it) => it,
        None => (cmd, ""),
    };

    tracing::debug!("executing command: {:?}", cmd_with_args);

    let mode_before = state.modes.current.clone();
    let mode = get_mode_after_command(&state.modes.previous);

    // NOTE: all file commands like e.g. d! should use preview path as target to enable cdo
    let result = match cmd_with_args {
        ("cdo", command) => add_change_mode(mode_before, mode, qfix::cdo(&mut state.qfix, command)),
        ("cfirst", "") => add_change_mode(mode_before, mode, qfix::select_first(&mut state.qfix)),
        ("cl", "") => print::qfix(&state.qfix),
        ("clearcl", "") => add_change_mode(
            mode_before,
            mode,
            qfix::reset(&mut state.qfix, app.contents.buffers.values_mut().collect()),
        ),
        ("clearcl", path) => add_change_mode(
            mode_before,
            mode,
            qfix::clear_in(app, &mut state.qfix, path),
        ),
        ("cn", "") => add_change_mode(mode_before, mode, qfix::next(&mut state.qfix)),
        ("cN", "") => add_change_mode(mode_before, mode, qfix::previous(&mut state.qfix)),
        ("cp", target) => {
            let path = get_preview_path(app);
            let actions = match path {
                Some(source_path) => file::copy_path(&state.marks, source_path, target),
                None => {
                    tracing::warn!("cp command failed: no path in preview buffer");
                    Vec::new()
                }
            };

            add_change_mode(mode_before, mode, actions)
        }
        ("d!", "") => {
            let path = get_preview_path(app);
            let actions = match path {
                Some(path) => {
                    tracing::info!("deleting path: {:?}", path);

                    vec![Action::Task(Task::DeletePath(path.to_path_buf()))]
                }
                _ => {
                    tracing::warn!("deleting path failed: no path in preview set");

                    vec![Action::EmitMessages(vec![Message::Error(
                        "No path in preview buffer to delete".to_string(),
                    )])]
                }
            };

            add_change_mode(mode_before, mode, actions)
        }
        ("delm", args) if !args.is_empty() => {
            let mut marks = Vec::new();
            for mark in args.chars().filter(|c| c != &' ') {
                marks.push(mark);
            }

            add_change_mode(
                mode_before,
                mode,
                vec![action::emit_keymap(KeymapMessage::DeleteMarks(marks))],
            )
        }
        ("delt", args) if !args.is_empty() => {
            let actions = match args.parse::<u16>() {
                Ok(it) => task::delete(&mut state.tasks, it),
                Err(err) => {
                    tracing::warn!("Failed to parse id: {}", err);
                    return Vec::new();
                }
            };

            add_change_mode(mode_before, mode, actions)
        }
        ("e!", "") => add_change_mode(mode_before, mode, file::refresh(app)),
        ("fd", params) => {
            let current_path = get_current_path(app);
            let actions = match current_path {
                Some(path) => vec![Action::Task(Task::ExecuteFd(
                    path.to_path_buf(),
                    params.to_owned(),
                ))],
                None => vec![Action::EmitMessages(vec![Message::Error(
                    "Fd failed. Current path could not be resolved.".to_string(),
                )])],
            };

            add_change_mode(mode_before, mode, actions)
        }
        ("invertcl", "") => add_change_mode(
            mode_before,
            mode,
            qfix::invert_in_current(app, &mut state.qfix),
        ),
        ("junk", "") => print::junkyard(&state.junk),
        ("marks", "") => print::marks(&state.marks),
        ("mv", target) => {
            let preview_path = get_preview_path(app);
            let actions = match preview_path {
                Some(source_path) => file::rename_path(&state.marks, source_path, target),
                None => vec![Action::EmitMessages(vec![Message::Error(
                    "Mv failed. Preview path could not be resolved.".to_string(),
                )])],
            };

            add_change_mode(mode_before, mode, actions)
        }
        ("noh", "") => add_change_mode(
            mode_before,
            mode,
            vec![Action::EmitMessages(vec![Message::Keymap(
                KeymapMessage::ClearSearchHighlight,
            )])],
        ),
        ("q", "") => {
            let buffer_id = match app.current_window() {
                Ok(window) => window.focused_viewport().buffer_id,
                Err(_) => return Vec::new(),
            };
            if has_unsaved_changes(&app.contents, Some(buffer_id)) {
                return print_error(
                    "No write since last change (add ! to override)",
                    mode_before,
                    mode,
                );
            }
            close_focused_window_or_quit(app, QuitMode::FailOnRunningTasks, mode_before, false)
        }
        ("q!", "") => close_focused_window_or_quit(app, QuitMode::Force, mode_before, true),
        ("qa", "") => {
            if has_unsaved_changes(&app.contents, None) {
                return print_error(
                    "No write since last change (add ! to override)",
                    mode_before,
                    mode,
                );
            }
            vec![action::emit_keymap(KeymapMessage::Quit(
                QuitMode::FailOnRunningTasks,
            ))]
        }
        ("qa!", "") => vec![action::emit_keymap(KeymapMessage::Quit(QuitMode::Force))],
        ("reg", "") => print::register(&state.register),
        ("rg", params) => {
            let current_path = get_current_path(app);
            let actions = match current_path {
                Some(path) => {
                    tracing::info!("executing rg in path: {:?}", path);

                    vec![Action::Task(Task::ExecuteRg(
                        path.to_path_buf(),
                        params.to_owned(),
                    ))]
                }
                None => {
                    vec![Action::EmitMessages(vec![Message::Error(
                        "Rg failed. Current path could not be resolved.".to_string(),
                    )])]
                }
            };

            add_change_mode(mode_before, mode, actions)
        }
        ("split", args) => {
            let preview_path = get_current_path(app);
            match preview_path {
                Some(path) => {
                    let target_path = match file::expand_path(&state.marks, args.trim(), path) {
                        Ok(target_path) => target_path,
                        Err(err) => return vec![Action::EmitMessages(vec![Message::Error(err)])],
                    };

                    add_change_mode(
                        mode_before,
                        Mode::Navigation,
                        split::horizontal(app, target_path.as_path()),
                    )
                }
                None => vec![Action::EmitMessages(vec![Message::Error(
                    "Split failed. Preview path could not be resolved.".to_string(),
                )])],
            }
        }
        ("tl", "") => print::tasks(&state.tasks),
        ("tabnew", "") => {
            let actions = match tab::tabnew_target_path(app) {
                Ok(path) => tab::create_tab(app, path.as_path()),
                Err(err) => vec![Action::EmitMessages(vec![Message::Error(err.to_string())])],
            };
            add_change_mode(mode_before, Mode::Navigation, actions)
        }
        ("tabc", "") => {
            if app.tabs.len() <= 1 {
                return vec![action::emit_keymap(KeymapMessage::Quit(
                    QuitMode::FailOnRunningTasks,
                ))];
            }
            tab::close_tab(app);
            add_change_mode(mode_before, Mode::Navigation, Vec::new())
        }
        ("tabo", "") => {
            tab::close_other_tabs(app);
            add_change_mode(mode_before, Mode::Navigation, Vec::new())
        }
        ("tabfir", "") => {
            tab::first_tab(app);
            add_change_mode(mode_before, Mode::Navigation, Vec::new())
        }
        ("tabl", "") => {
            tab::last_tab(app);
            add_change_mode(mode_before, Mode::Navigation, Vec::new())
        }
        ("tabn", "") => {
            tab::next_tab(app);
            add_change_mode(mode_before, Mode::Navigation, Vec::new())
        }
        ("tabp", "") => {
            tab::previous_tab(app);
            add_change_mode(mode_before, Mode::Navigation, Vec::new())
        }
        ("topen", "") => {
            add_change_mode(mode_before, Mode::Navigation, task::open(app, &state.tasks))
        }
        ("vsplit", args) => {
            let preview_path = get_current_path(app);
            match preview_path {
                Some(path) => {
                    let target_path = match file::expand_path(&state.marks, args.trim(), path) {
                        Ok(target_path) => target_path,
                        Err(err) => return vec![Action::EmitMessages(vec![Message::Error(err)])],
                    };

                    add_change_mode(
                        mode_before,
                        Mode::Navigation,
                        split::vertical(app, target_path.as_path()),
                    )
                }
                None => vec![Action::EmitMessages(vec![Message::Error(
                    "Vsplit failed. Preview path could not be resolved.".to_string(),
                )])],
            }
        }
        ("w", "") => add_change_mode(
            mode_before,
            mode,
            vec![Action::EmitMessages(vec![Message::Keymap(
                KeymapMessage::Buffer(BufferMessage::SaveBuffer),
            )])],
        ),
        ("wq", "") => {
            close_focused_window_or_quit(app, QuitMode::FailOnRunningTasks, mode_before, false)
        }
        ("z", params) => add_change_mode(
            mode_before,
            mode,
            vec![Action::Task(Task::ExecuteZoxide(params.to_owned()))],
        ),
        (cmd, args) => {
            let mut actions = Vec::new();
            if !args.is_empty() {
                let err = format!("command '{} {}' is not valid", cmd, args);
                actions.push(Action::EmitMessages(vec![Message::Error(err)]));
            }
            add_change_mode(mode_before, mode, actions)
        }
    };

    state.register.command = Some(cmd.to_string());

    result
}

fn get_current_path(app: &App) -> Option<&Path> {
    let window = app.current_window().ok()?;
    let (_, current_id, _) = app::get_focused_directory_buffer_ids(window)?;
    app::get_buffer_path(app, current_id)
}

fn get_preview_path(app: &App) -> Option<&Path> {
    let window = app.current_window().ok()?;
    let (_, _, preview_id) = app::get_focused_directory_buffer_ids(window)?;
    app::get_buffer_path(app, preview_id)
}

fn add_change_mode(mode_before: Mode, mode: Mode, mut actions: Vec<Action>) -> Vec<Action> {
    let emit = actions.iter_mut().find_map(|action| {
        if let Action::EmitMessages(messages) = action {
            Some(messages)
        } else {
            None
        }
    });

    let change_mode_message = Message::Keymap(KeymapMessage::Buffer(BufferMessage::ChangeMode(
        mode_before,
        mode,
    )));

    match emit {
        Some(messages) => messages.insert(0, change_mode_message),
        None => actions.insert(0, Action::EmitMessages(vec![change_mode_message])),
    }

    actions
}

fn has_unsaved_changes(contents: &Contents, buffer_id: Option<usize>) -> bool {
    contents.buffers.iter().any(|(key, buf)| {
        if let Some(id) = buffer_id {
            if *key != id {
                return false;
            }
        }

        if let Buffer::Directory(dir) = buf {
            !dir.buffer.undo.get_uncommited_changes().is_empty()
        } else {
            false
        }
    })
}

fn reset_unsaved_changes(window: &Window, contents: &mut Contents) {
    for buffer_id in window.buffer_ids() {
        if let Some(Buffer::Directory(dir)) = contents.buffers.get_mut(&buffer_id) {
            dir.buffer.undo.reset_to_last_save();
        }
    }
}

fn close_focused_window_or_quit(
    app: &mut App,
    quit_mode: QuitMode,
    mode_before: Mode,
    discard_changes: bool,
) -> Vec<Action> {
    let (window, contents) = match app.current_window_and_contents_mut() {
        Ok(window) => window,
        Err(_) => return Vec::new(),
    };
    let old_window = mem::take(window);
    match old_window {
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
            let (kept, dropped) = match focus {
                SplitFocus::First => (*second, *first),
                SplitFocus::Second => (*first, *second),
            };

            if discard_changes {
                reset_unsaved_changes(&dropped, contents);
            }

            *window = kept;
            add_change_mode(mode_before, Mode::Navigation, Vec::new())
        }
        other => {
            *window = other;
            vec![action::emit_keymap(KeymapMessage::Quit(quit_mode))]
        }
    }
}

fn print_error(msg: &str, mode_before: Mode, mode: Mode) -> Vec<Action> {
    add_change_mode(
        mode_before,
        mode,
        vec![Action::EmitMessages(vec![Message::Error(msg.to_string())])],
    )
}

fn get_mode_after_command(mode_before: &Option<Mode>) -> Mode {
    if let Some(mode) = mode_before {
        match mode {
            Mode::Command(_) => Mode::default(),
            Mode::Insert | Mode::Normal => Mode::Normal,
            Mode::Navigation => Mode::Navigation,
        }
    } else {
        Mode::default()
    }
}

#[cfg(test)]
mod test {
    use yeet_buffer::model::{
        ansi::Ansi, undo::BufferChanged, viewport::ViewPort, CommandMode, Mode,
    };
    use yeet_keymap::message::{KeymapMessage, QuitMode};

    use crate::{
        action::Action,
        event::Message,
        model::{App, Buffer, Contents, DirectoryBuffer, SplitFocus, State, TasksBuffer, Window},
    };

    use super::execute;

    fn make_state_with_command_mode() -> State {
        let mut state = State::default();
        state.modes.current = Mode::Command(CommandMode::Command);
        state.modes.previous = Some(Mode::Navigation);
        state
    }

    fn make_app_with_horizontal_split() -> App {
        let mut app = App::default();
        let tasks_buffer_id = 100;
        app.contents
            .buffers
            .insert(tasks_buffer_id, Buffer::Tasks(TasksBuffer::default()));

        let window = app.current_window_mut().expect("test requires current tab");
        let old_window = std::mem::take(window);
        *window = Window::Horizontal {
            first: Box::new(old_window),
            second: Box::new(Window::Tasks(ViewPort {
                buffer_id: tasks_buffer_id,
                ..Default::default()
            })),
            focus: SplitFocus::Second,
        };
        app
    }

    fn make_app_with_unsaved_changes() -> App {
        let mut app = App::default();
        // Create a directory buffer with unsaved changes
        let mut dir_buffer = DirectoryBuffer::default();
        dir_buffer.buffer.undo.add(
            &Mode::Normal,
            vec![BufferChanged::LineAdded(0, Ansi::new("test"))],
        );
        app.contents
            .buffers
            .insert(50, Buffer::Directory(dir_buffer));
        app
    }

    fn make_app_with_unsaved_changes_and_split() -> App {
        let mut app = make_app_with_unsaved_changes();
        let tasks_buffer_id = 100;
        app.contents
            .buffers
            .insert(tasks_buffer_id, Buffer::Tasks(TasksBuffer::default()));

        let window = app.current_window_mut().expect("test requires current tab");
        let old_window = std::mem::take(window);
        *window = Window::Horizontal {
            first: Box::new(old_window),
            second: Box::new(Window::Tasks(ViewPort {
                buffer_id: tasks_buffer_id,
                ..Default::default()
            })),
            focus: SplitFocus::Second,
        };
        app
    }

    fn contains_quit_action(actions: &[Action], expected_mode: &QuitMode) -> bool {
        actions.iter().any(|a| {
            if let Action::EmitMessages(msgs) = a {
                msgs.iter().any(|m| {
                    matches!(m, Message::Keymap(KeymapMessage::Quit(mode)) if mode == expected_mode)
                })
            } else {
                false
            }
        })
    }

    fn contains_error_message(actions: &[Action]) -> bool {
        actions.iter().any(|a| {
            if let Action::EmitMessages(msgs) = a {
                msgs.iter().any(
                    |m| matches!(m, Message::Error(s) if s.contains("No write since last change")),
                )
            } else {
                false
            }
        })
    }

    fn contains_command_error(actions: &[Action], needle: &str) -> bool {
        actions.iter().any(|a| {
            if let Action::EmitMessages(msgs) = a {
                msgs.iter()
                    .any(|m| matches!(m, Message::Error(s) if s.contains(needle)))
            } else {
                false
            }
        })
    }

    fn contains_change_mode(actions: &[Action], from: &Mode, to: &Mode) -> bool {
        use yeet_buffer::message::BufferMessage;
        actions.iter().any(|a| {
            if let Action::EmitMessages(msgs) = a {
                msgs.iter().any(|m| {
                    matches!(
                        m,
                        Message::Keymap(KeymapMessage::Buffer(BufferMessage::ChangeMode(f, t)))
                            if f == from && t == to
                    )
                })
            } else {
                false
            }
        })
    }

    #[test]
    fn get_mode_after_command() {
        let mode_before = Some(Mode::Normal);
        let result = super::get_mode_after_command(&mode_before);
        assert_eq!(result, Mode::Normal);

        let mode_before = Some(Mode::Insert);
        let result = super::get_mode_after_command(&mode_before);
        assert_eq!(result, Mode::Normal);

        let mode_before = Some(Mode::Navigation);
        let result = super::get_mode_after_command(&mode_before);
        assert_eq!(result, Mode::Navigation);

        let mode_before = None;
        let result = super::get_mode_after_command(&mode_before);
        assert_eq!(result, Mode::Navigation);
    }

    #[test]
    fn q_on_horizontal_closes_focused_and_collapses_to_directory() {
        let mut app = make_app_with_horizontal_split();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Directory(_, _, _)));
        assert!(!contains_quit_action(
            &actions,
            &QuitMode::FailOnRunningTasks
        ));
    }

    #[test]
    fn q_on_horizontal_emits_change_mode_to_navigation() {
        let mut app = make_app_with_horizontal_split();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Directory(_, _, _)));
        assert!(
            contains_change_mode(
                &actions,
                &Mode::Command(CommandMode::Command),
                &Mode::Navigation,
            ),
            "closing a split via :q must emit ChangeMode(Command, Navigation) so the app leaves command mode; actions: {:?}",
            actions,
        );
    }

    #[test]
    fn q_on_horizontal_focus_first_closes_directory_keeps_tasks() {
        let mut app = make_app_with_horizontal_split();
        // Switch focus to First (directory)
        let window = app.current_window_mut().expect("test requires current tab");
        if let Window::Horizontal { focus, .. } = window {
            *focus = SplitFocus::First;
        }
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Tasks(_)));
        assert!(!contains_quit_action(
            &actions,
            &QuitMode::FailOnRunningTasks
        ));
    }

    #[test]
    fn q_on_single_directory_emits_quit() {
        let mut app = App::default();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        assert!(contains_quit_action(
            &actions,
            &QuitMode::FailOnRunningTasks
        ));
    }

    #[test]
    fn q_with_unsaved_changes_prints_error() {
        let mut app = make_app_with_unsaved_changes();
        // Point the focused viewport (current/middle) at the dirty buffer (id 50)
        let window = app.current_window_mut().expect("test requires current tab");
        if let Window::Directory(_, current_vp, _) = window {
            current_vp.buffer_id = 50;
        }
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        assert!(contains_error_message(&actions));
        // Window should remain unchanged
        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Directory(_, _, _)));
    }

    #[test]
    fn q_on_split_focused_on_dirty_buffer_prints_error() {
        // Unsaved changes in buffer 50 (directory in first child).
        // Focus is switched to First (directory) so :q checks that buffer.
        let mut app = make_app_with_unsaved_changes_and_split();
        let window = app.current_window_mut().expect("test requires current tab");
        if let Window::Horizontal { focus, .. } = window {
            *focus = SplitFocus::First;
        }
        // The focused directory viewport points at buffer_id 1 (from App::default),
        // but the dirty buffer is at id 50. We need to make the focused viewport
        // point at the dirty buffer to trigger the check.
        let window = app.current_window_mut().expect("test requires current tab");
        if let Window::Horizontal { first, .. } = window {
            if let Window::Directory(_, current_vp, _) = first.as_mut() {
                current_vp.buffer_id = 50;
            }
        }
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        assert!(contains_error_message(&actions));
        // Window should remain a Horizontal split
        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Horizontal { .. }));
    }

    #[test]
    fn q_bang_on_horizontal_closes_without_unsaved_check() {
        let mut app = make_app_with_unsaved_changes_and_split();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q!");

        assert!(!contains_error_message(&actions));
        // Should have collapsed the split
        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Directory(_, _, _)));
    }

    #[test]
    fn q_bang_on_single_window_force_quits() {
        let mut app = make_app_with_unsaved_changes();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q!");

        assert!(contains_quit_action(&actions, &QuitMode::Force));
    }

    #[test]
    fn qa_emits_quit_even_in_split() {
        let mut app = make_app_with_horizontal_split();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "qa");

        assert!(contains_quit_action(
            &actions,
            &QuitMode::FailOnRunningTasks
        ));
    }

    #[test]
    fn qa_with_unsaved_changes_prints_error() {
        let mut app = make_app_with_unsaved_changes();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "qa");

        assert!(contains_error_message(&actions));
        assert!(!contains_quit_action(
            &actions,
            &QuitMode::FailOnRunningTasks
        ));
    }

    #[test]
    fn qa_bang_force_quits() {
        let mut app = App::default();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "qa!");

        assert!(contains_quit_action(&actions, &QuitMode::Force));
    }

    #[test]
    fn qa_bang_with_unsaved_changes_still_quits() {
        let mut app = make_app_with_unsaved_changes();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "qa!");

        assert!(contains_quit_action(&actions, &QuitMode::Force));
    }

    #[test]
    fn buffer_cleanup_preserves_kept_window_buffers() {
        let mut app = make_app_with_horizontal_split();
        let mut state = make_state_with_command_mode();

        // Collect buffer ids from the first child (directory) before closing
        let window = app.current_window().expect("test requires current tab");
        let dir_buffer_ids: Vec<usize> = match window {
            Window::Horizontal { first, .. } => first.buffer_ids().into_iter().collect(),
            _ => panic!("expected Horizontal"),
        };

        execute(&mut app, &mut state, "q");

        // All directory buffer ids should still exist
        for id in &dir_buffer_ids {
            assert!(
                app.contents.buffers.contains_key(id),
                "buffer {} should be preserved",
                id
            );
        }
    }

    #[test]
    fn has_unsaved_changes_returns_false_for_clean_buffers() {
        let contents = Contents {
            buffers: {
                let mut map = std::collections::HashMap::new();
                map.insert(1, Buffer::Directory(DirectoryBuffer::default()));
                map.insert(2, Buffer::Tasks(TasksBuffer::default()));
                map.insert(3, Buffer::Empty);
                map
            },
            latest_buffer_id: 3,
        };
        assert!(!super::has_unsaved_changes(&contents, None));
    }

    #[test]
    fn has_unsaved_changes_returns_true_for_dirty_buffer() {
        let mut dir_buffer = DirectoryBuffer::default();
        dir_buffer.buffer.undo.add(
            &Mode::Normal,
            vec![BufferChanged::LineAdded(0, Ansi::new("test"))],
        );

        let contents = Contents {
            buffers: {
                let mut map = std::collections::HashMap::new();
                map.insert(1, Buffer::Directory(dir_buffer));
                map
            },
            latest_buffer_id: 1,
        };
        assert!(super::has_unsaved_changes(&contents, None));
    }

    #[test]
    fn has_unsaved_changes_ignores_non_directory_buffers() {
        let contents = Contents {
            buffers: {
                let mut map = std::collections::HashMap::new();
                map.insert(1, Buffer::Tasks(TasksBuffer::default()));
                map.insert(2, Buffer::Empty);
                map
            },
            latest_buffer_id: 2,
        };
        assert!(!super::has_unsaved_changes(&contents, None));
    }

    #[test]
    fn has_unsaved_changes_checks_only_specified_buffer() {
        let mut dirty_buffer = DirectoryBuffer::default();
        dirty_buffer.buffer.undo.add(
            &Mode::Normal,
            vec![BufferChanged::LineAdded(0, Ansi::new("test"))],
        );

        let contents = Contents {
            buffers: {
                let mut map = std::collections::HashMap::new();
                map.insert(1, Buffer::Directory(DirectoryBuffer::default()));
                map.insert(2, Buffer::Directory(dirty_buffer));
                map
            },
            latest_buffer_id: 2,
        };

        // Checking buffer 1 (clean) should return false
        assert!(!super::has_unsaved_changes(&contents, Some(1)));
        // Checking buffer 2 (dirty) should return true
        assert!(super::has_unsaved_changes(&contents, Some(2)));
        // Checking all (None) should return true
        assert!(super::has_unsaved_changes(&contents, None));
    }

    #[test]
    fn tabnew_creates_new_tab_and_sets_current() {
        let mut app = App::default();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "tabnew");

        assert_eq!(app.tabs.len(), 2);
        assert!(app.tabs.contains_key(&2));
        assert_eq!(app.current_tab_id, 2);
        assert!(actions.iter().any(|action| match action {
            Action::EmitMessages(messages) => messages.iter().any(|message| {
                matches!(message, Message::Keymap(KeymapMessage::NavigateToPath(_)))
            }),
            _ => false,
        }));
    }

    #[test]
    fn tabc_on_last_tab_emits_quit() {
        let mut app = App::default();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "tabc");

        assert!(contains_quit_action(
            &actions,
            &QuitMode::FailOnRunningTasks
        ));
    }

    #[test]
    fn tabn_and_tabp_wrap_across_tabs() {
        let mut app = App::default();
        let mut state = make_state_with_command_mode();
        execute(&mut app, &mut state, "tabnew");
        execute(&mut app, &mut state, "tabnew");
        assert_eq!(app.current_tab_id, 3);

        let _ = execute(&mut app, &mut state, "tabn");
        assert_eq!(app.current_tab_id, 1);

        let _ = execute(&mut app, &mut state, "tabp");
        assert_eq!(app.current_tab_id, 3);
    }

    #[test]
    fn tabnew_uses_home_when_tasks_focused() {
        let mut app = App::default();
        let window = app.current_window_mut().expect("test requires current tab");
        *window = Window::Tasks(ViewPort::default());
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "tabnew");

        if dirs::home_dir().is_some() {
            assert!(actions.iter().any(|action| match action {
                Action::EmitMessages(messages) => messages.iter().any(|message| {
                    matches!(message, Message::Keymap(KeymapMessage::NavigateToPath(_)))
                }),
                _ => false,
            }));
        } else {
            assert!(contains_command_error(
                &actions,
                "Tabnew failed. Target path could not be resolved."
            ));
        }
    }

    #[test]
    fn q_on_split_with_unsaved_in_other_window_closes() {
        // Unsaved changes are in buffer 50 (directory), but focus is on buffer 100 (tasks).
        // :q should only check the focused buffer, so it should close the split.
        let mut app = make_app_with_unsaved_changes_and_split();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        assert!(!contains_error_message(&actions));
        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Directory(_, _, _)));
    }

    #[test]
    fn q_bang_on_split_resets_dropped_pane_undo() {
        // Setup: split with focus on First (Directory that has dirty buffer 50).
        // :q! drops the focused First pane. The dirty buffer's undo should be reset
        // so that the subsequent ChangeMode → Navigation → save::all doesn't persist
        // the discarded changes.
        let mut app = make_app_with_unsaved_changes_and_split();
        let window = app.current_window_mut().expect("test requires current tab");
        if let Window::Horizontal { focus, first, .. } = window {
            *focus = SplitFocus::First;
            // Point the focused directory viewport at the dirty buffer
            if let Window::Directory(_, current_vp, _) = first.as_mut() {
                current_vp.buffer_id = 50;
            }
        }
        let mut state = make_state_with_command_mode();

        // Verify the buffer is dirty before :q!
        if let Some(Buffer::Directory(dir)) = app.contents.buffers.get(&50) {
            assert!(
                !dir.buffer.undo.get_uncommited_changes().is_empty(),
                "buffer 50 should be dirty before :q!"
            );
        } else {
            panic!("buffer 50 should be a Directory");
        }

        let _actions = execute(&mut app, &mut state, "q!");

        // After :q!, the dropped pane's buffer undo should be reset
        if let Some(Buffer::Directory(dir)) = app.contents.buffers.get(&50) {
            assert!(
                dir.buffer.undo.get_uncommited_changes().is_empty(),
                "buffer 50 undo should be reset after :q! drops its pane"
            );
        }
        // The window should have collapsed to the kept pane (Tasks)
        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Tasks(_)));
    }

    #[test]
    fn q_bang_on_split_preserves_kept_pane_undo() {
        // Setup: split where both panes have dirty buffers.
        // Focus is on Second (Tasks), so Second is dropped and First (Directory) is kept.
        // The kept pane's directory buffer (id 50) should retain its unsaved changes.
        let mut app = make_app_with_unsaved_changes_and_split();
        // Focus is already SplitFocus::Second (from make_app_with_unsaved_changes_and_split)
        let mut state = make_state_with_command_mode();

        // Verify the buffer is dirty before :q!
        if let Some(Buffer::Directory(dir)) = app.contents.buffers.get(&50) {
            assert!(!dir.buffer.undo.get_uncommited_changes().is_empty());
        }

        let _actions = execute(&mut app, &mut state, "q!");

        // Buffer 50 is in the KEPT pane (First/Directory) — its changes should be preserved
        if let Some(Buffer::Directory(dir)) = app.contents.buffers.get(&50) {
            assert!(
                !dir.buffer.undo.get_uncommited_changes().is_empty(),
                "kept pane's buffer 50 should still have unsaved changes after :q!"
            );
        } else {
            panic!("buffer 50 should still exist and be a Directory");
        }
        // Window should have collapsed to Directory (the first/kept pane)
        let window = app.current_window().expect("test requires current tab");
        assert!(matches!(window, Window::Directory(_, _, _)));
    }

    #[test]
    fn wq_on_horizontal_from_normal_emits_change_mode_to_navigation() {
        // When the user was in Normal mode (e.g. after editing a buffer line and pressing Esc),
        // then enters command mode with ":" and runs ":wq", the mode after closing the split
        // must be Navigation -- not Normal. Transitioning to Navigation triggers save::all,
        // which commits all directory buffer changes to filesystem tasks.
        let mut app = make_app_with_horizontal_split();
        let mut state = State::default();
        state.modes.current = Mode::Command(CommandMode::Command);
        state.modes.previous = Some(Mode::Normal);

        let actions = execute(&mut app, &mut state, "wq");

        let window = app.current_window().expect("test requires current tab");
        assert!(
            matches!(window, Window::Directory(_, _, _)),
            "wq on split must collapse to Directory",
        );

        assert!(
            contains_change_mode(
                &actions,
                &Mode::Command(CommandMode::Command),
                &Mode::Navigation,
            ),
            "wq must change mode to Navigation (not Normal) so save::all runs; actions: {actions:?}",
        );

        assert!(
            !contains_quit_action(&actions, &QuitMode::FailOnRunningTasks),
            "wq on split should close the pane, not quit the app; actions: {actions:?}",
        );
    }
}
