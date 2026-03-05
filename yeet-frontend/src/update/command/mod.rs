use std::{mem, path::Path};

use yeet_buffer::{message::BufferMessage, model::Mode};
use yeet_keymap::message::{KeymapMessage, QuitMode};

use crate::{
    action::{self, Action},
    event::Message,
    model::{App, Buffer, Contents, SplitFocus, State, Window},
    task::Task,
    update::app,
};

mod file;
mod print;
mod qfix;
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
            if has_unsaved_changes(&app.contents) {
                return print_error(
                    "No write since last change (add ! to override)",
                    mode_before,
                    mode,
                );
            }
            close_focused_window_or_quit(app, QuitMode::FailOnRunningTasks, mode_before, mode)
        }
        ("q!", "") => close_focused_window_or_quit(app, QuitMode::Force, mode_before, mode),
        ("qa", "") => {
            if has_unsaved_changes(&app.contents) {
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
        ("tl", "") => print::tasks(&state.tasks),
        ("topen", "") => {
            add_change_mode(mode_before, Mode::Navigation, task::open(app, &state.tasks))
        }
        ("w", "") => add_change_mode(
            mode_before,
            mode,
            vec![Action::EmitMessages(vec![Message::Keymap(
                KeymapMessage::Buffer(BufferMessage::SaveBuffer),
            )])],
        ),
        ("wq", "") => add_change_mode(
            mode_before,
            mode,
            vec![Action::EmitMessages(vec![
                Message::Keymap(KeymapMessage::Buffer(BufferMessage::SaveBuffer)),
                Message::Keymap(KeymapMessage::Quit(QuitMode::FailOnRunningTasks)),
            ])],
        ),
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
    let (_, current_id, _) = app::get_focused_directory_buffer_ids(&app.window)?;
    app::get_buffer_path(app, current_id)
}

fn get_preview_path(app: &App) -> Option<&Path> {
    let (_, _, preview_id) = app::get_focused_directory_buffer_ids(&app.window)?;
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

fn has_unsaved_changes(contents: &Contents) -> bool {
    contents.buffers.values().any(|buf| {
        if let Buffer::Directory(dir) = buf {
            !dir.buffer.undo.get_uncommited_changes().is_empty()
        } else {
            false
        }
    })
}

fn close_focused_window_or_quit(
    app: &mut App,
    quit_mode: QuitMode,
    mode_before: Mode,
    mode: Mode,
) -> Vec<Action> {
    let old_window = mem::take(&mut app.window);
    match old_window {
        Window::Horizontal {
            first,
            second,
            focus,
        } => {
            let (kept, closed) = match focus {
                SplitFocus::First => (*second, *first),
                SplitFocus::Second => (*first, *second),
            };
            cleanup_window_buffers(&mut app.contents, &kept, &closed);
            app.window = kept;
            add_change_mode(mode_before, mode, Vec::new())
        }
        other => {
            app.window = other;
            vec![action::emit_keymap(KeymapMessage::Quit(quit_mode))]
        }
    }
}

fn cleanup_window_buffers(contents: &mut Contents, kept: &Window, closed: &Window) {
    let kept_ids = kept.buffer_ids();
    for id in closed.buffer_ids() {
        if !kept_ids.contains(&id) {
            contents.buffers.remove(&id);
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

        let old_window = std::mem::take(&mut app.window);
        app.window = Window::Horizontal {
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

        let old_window = std::mem::take(&mut app.window);
        app.window = Window::Horizontal {
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

        assert!(matches!(app.window, Window::Directory(_, _, _)));
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

        assert!(matches!(app.window, Window::Directory(_, _, _)));
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
    fn q_on_horizontal_removes_task_buffer_from_contents() {
        let mut app = make_app_with_horizontal_split();
        let mut state = make_state_with_command_mode();

        // The task buffer is at id 100
        assert!(app.contents.buffers.contains_key(&100));

        execute(&mut app, &mut state, "q");

        assert!(!app.contents.buffers.contains_key(&100));
    }

    #[test]
    fn q_on_horizontal_focus_first_closes_directory_keeps_tasks() {
        let mut app = make_app_with_horizontal_split();
        // Switch focus to First (directory)
        if let Window::Horizontal { focus, .. } = &mut app.window {
            *focus = SplitFocus::First;
        }
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        assert!(matches!(app.window, Window::Tasks(_)));
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
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        assert!(contains_error_message(&actions));
        // Window should remain unchanged
        assert!(matches!(app.window, Window::Directory(_, _, _)));
    }

    #[test]
    fn q_with_unsaved_changes_on_split_prints_error_does_not_close() {
        let mut app = make_app_with_unsaved_changes_and_split();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q");

        assert!(contains_error_message(&actions));
        // Window should remain a Horizontal split
        assert!(matches!(app.window, Window::Horizontal { .. }));
    }

    #[test]
    fn q_bang_on_horizontal_closes_without_unsaved_check() {
        let mut app = make_app_with_unsaved_changes_and_split();
        let mut state = make_state_with_command_mode();

        let actions = execute(&mut app, &mut state, "q!");

        assert!(!contains_error_message(&actions));
        // Should have collapsed the split
        assert!(matches!(app.window, Window::Directory(_, _, _)));
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
        let dir_buffer_ids: Vec<usize> = match &app.window {
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
        assert!(!super::has_unsaved_changes(&contents));
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
        assert!(super::has_unsaved_changes(&contents));
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
        assert!(!super::has_unsaved_changes(&contents));
    }
}
