use std::path::Path;

use yeet_buffer::{message::BufferMessage, model::Mode};
use yeet_keymap::message::{KeymapMessage, QuitMode};

use crate::{
    action::{self, Action},
    event::Message,
    model::{App, Buffer, State},
    task::Task,
    update::app,
};

mod file;
mod print;
mod qfix;
mod task;

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
            qfix::reset(&mut state.qfix, app.buffers.values_mut().collect()),
        ),
        ("clearcl", path) => add_change_mode(
            mode_before,
            mode,
            qfix::clear_in(app, &mut state.qfix, path),
        ),
        ("cn", "") => add_change_mode(mode_before, mode, qfix::next(&mut state.qfix)),
        ("cN", "") => add_change_mode(mode_before, mode, qfix::previous(&mut state.qfix)),
        ("cp", target) => {
            let (_, _, preview_id) = app::directory_buffer_ids(app);
            let path = match app.buffers.get(&preview_id) {
                Some(Buffer::Directory(it)) => it.resolve_path(),
                Some(Buffer::Content(it)) => it.resolve_path(),
                Some(Buffer::Image(it)) => it.resolve_path(),
                Some(Buffer::PathReference(path)) => Some(path.as_path()),
                Some(Buffer::Empty) | None => None,
            };

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
        ("q", "") => vec![action::emit_keymap(KeymapMessage::Quit(
            QuitMode::FailOnRunningTasks,
        ))],
        ("q!", "") => vec![action::emit_keymap(KeymapMessage::Quit(QuitMode::Force))],
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
    let (_, current_id, _) = app::directory_buffer_ids(app);
    get_buffer_path(app, current_id)
}

fn get_preview_path(app: &App) -> Option<&Path> {
    let (_, _, preview_id) = app::directory_buffer_ids(app);
    get_buffer_path(app, preview_id)
}

fn get_buffer_path(app: &App, buffer_id: usize) -> Option<&Path> {
    match app.buffers.get(&buffer_id) {
        Some(Buffer::Directory(it)) => it.resolve_path(),
        Some(Buffer::Content(it)) => it.resolve_path(),
        Some(Buffer::Image(it)) => it.resolve_path(),
        Some(Buffer::PathReference(path)) => Some(path.as_path()),
        Some(Buffer::Empty) | None => None,
    }
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

mod test {
    #[test]
    fn get_mode_after_command() {
        use yeet_buffer::model::Mode;

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
}
