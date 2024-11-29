use yeet_buffer::{message::BufferMessage, model::Mode};
use yeet_keymap::message::{KeymapMessage, QuitMode};

use crate::{
    action::{self, Action},
    event::Message,
    model::Model,
    task::Task,
};

mod file;
mod print;
mod qfix;
mod task;

#[tracing::instrument(skip(model))]
pub fn execute(cmd: &str, model: &mut Model) -> Vec<Action> {
    let cmd_with_args = match cmd.split_once(' ') {
        Some(it) => it,
        None => (cmd, ""),
    };

    tracing::debug!("executing command: {:?}", cmd_with_args);

    let mode_before = model.mode.clone();
    let mode = get_mode_after_command(&model.mode_before);

    // NOTE: all file commands like e.g. d! should use preview path as target to enable cdo
    match cmd_with_args {
        ("cdo", command) => add_change_mode(mode_before, mode, qfix::cdo(model, command)),
        ("cfirst", "") => add_change_mode(mode_before, mode, qfix::select_first(model)),
        ("cl", "") => print::qfix(&model.qfix),
        ("clearcl", "") => add_change_mode(mode_before, mode, qfix::reset(model)),
        ("clearcl", path) => add_change_mode(mode_before, mode, qfix::clear_in(model, path)),
        ("cn", "") => add_change_mode(mode_before, mode, qfix::next(model)),
        ("cN", "") => add_change_mode(mode_before, mode, qfix::previous(model)),
        ("cp", target) => add_change_mode(mode_before, mode, file::copy(model, target)),
        ("d!", "") => add_change_mode(mode_before, mode, file::delete_selection(model)),
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
                Ok(it) => task::delete(model, it),
                Err(err) => {
                    tracing::warn!("Failed to parse id: {}", err);
                    return Vec::new();
                }
            };

            add_change_mode(mode_before, mode, actions)
        }
        ("e!", "") => add_change_mode(mode_before, mode, file::refresh(model)),
        ("fd", params) => add_change_mode(
            mode_before,
            mode,
            vec![Action::Task(Task::ExecuteFd(
                model.files.current.path.clone(),
                params.to_owned(),
            ))],
        ),
        ("invertcl", "") => add_change_mode(mode_before, mode, qfix::invert_in_current(model)),
        ("junk", "") => print::junkyard(&model.junk),
        ("marks", "") => print::marks(&model.marks),
        ("mv", target) => add_change_mode(mode_before, mode, file::rename_selection(model, target)),
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
        ("reg", "") => print::register(&model.register),
        ("tl", "") => print::tasks(&model.current_tasks),
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
        (cmd, args) => {
            let mut actions = Vec::new();
            if !args.is_empty() {
                let err = format!("command '{} {}' is not valid", cmd, args);
                actions.push(Action::EmitMessages(vec![Message::Error(err)]));
            }
            add_change_mode(mode_before, mode, actions)
        }
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
