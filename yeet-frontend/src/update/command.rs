use yeet_keymap::message::{Buffer, Message, Mode, PrintContent};

use crate::{
    action::Action,
    model::Model,
    task::Task,
    update::{mark, qfix},
};

use super::current;

pub fn execute(cmd: &str, model: &mut Model) -> Vec<Action> {
    let change_mode_message = Message::Buffer(Buffer::ChangeMode(
        model.mode.clone(),
        get_mode_after_command(&model.mode_before),
    ));
    let change_mode_action = Action::EmitMessages(vec![change_mode_message.clone()]);

    let cmd = match cmd.split_once(' ') {
        Some(it) => it,
        None => (cmd, ""),
    };

    let mut actions = match cmd {
        ("cfirst", "") => {
            model.qfix.current_index = 0;

            let path = match model.qfix.entries.first() {
                Some(it) => it,
                None => return vec![change_mode_action],
            };

            vec![
                change_mode_action,
                Action::EmitMessages(vec![Message::NavigateToPath(path.clone())]),
            ]
        }
        ("cl", "") => {
            let content = qfix::print(&model.qfix)
                .iter()
                .map(|cntnt| PrintContent::Info(cntnt.to_string()))
                .collect();

            vec![Action::EmitMessages(vec![Message::Print(content)])]
        }
        ("cn", "") => {
            let next_index = model.qfix.current_index + 1;
            match model.qfix.entries.get(next_index) {
                Some(it) => {
                    model.qfix.current_index = next_index;
                    vec![
                        change_mode_action,
                        Action::EmitMessages(vec![Message::NavigateToPath(it.clone())]),
                    ]
                }
                None => {
                    vec![Action::EmitMessages(vec![Message::ExecuteCommandString(
                        "cfirst".to_string(),
                    )])]
                }
            }
        }
        ("cN", "") => {
            if model.qfix.entries.is_empty() {
                return vec![change_mode_action];
            }

            let next_index = if model.qfix.current_index > 0 {
                model.qfix.current_index - 1
            } else {
                model.qfix.entries.len() - 1
            };

            model.qfix.current_index = next_index;

            match model.qfix.entries.get(next_index) {
                Some(it) => {
                    vec![
                        change_mode_action,
                        Action::EmitMessages(vec![Message::NavigateToPath(it.clone())]),
                    ]
                }
                None => {
                    vec![Action::EmitMessages(vec![Message::ExecuteCommandString(
                        "cN".to_string(),
                    )])]
                }
            }
        }
        ("d!", "") => {
            let mut actions = vec![change_mode_action];
            if let Some(path) = current::selection(model) {
                actions.push(Action::Task(Task::DeletePath(path.clone())));
            }
            actions
        }
        ("delm", args) if !args.is_empty() => {
            let mut marks = Vec::new();
            for mark in args.chars().filter(|c| c != &' ') {
                marks.push(mark);
            }

            vec![
                change_mode_action,
                Action::EmitMessages(vec![Message::DeleteMarks(marks)]),
            ]
        }
        ("e!", "") => vec![Action::EmitMessages(vec![
            change_mode_message,
            Message::NavigateToPath(model.current.path.clone()),
        ])],
        ("histopt", "") => vec![change_mode_action, Action::Task(Task::OptimizeHistory)],
        ("jnk", "") => {
            let content = model
                .junk
                .print()
                .iter()
                .map(|cntnt| PrintContent::Info(cntnt.to_string()))
                .collect();

            vec![Action::EmitMessages(vec![Message::Print(content)])]
        }
        ("marks", "") => {
            let content = mark::print(&model.marks)
                .iter()
                .map(|cntnt| PrintContent::Info(cntnt.to_string()))
                .collect();

            vec![Action::EmitMessages(vec![Message::Print(content)])]
        }
        ("noh", "") => vec![Action::EmitMessages(vec![
            change_mode_message,
            Message::ClearSearchHighlight,
        ])],
        ("q", "") => vec![Action::EmitMessages(vec![Message::Quit])],
        ("w", "") => vec![Action::EmitMessages(vec![
            change_mode_message,
            Message::Buffer(Buffer::SaveBuffer(None)),
        ])],
        ("wq", "") => vec![Action::EmitMessages(vec![
            Message::Buffer(Buffer::SaveBuffer(None)),
            Message::Quit,
        ])],
        (cmd, args) => {
            let mut actions = vec![change_mode_action];
            if !args.is_empty() {
                let err = format!("command '{} {}' is not valid", cmd, args);
                actions.push(Action::EmitMessages(vec![Message::Error(err)]));
            }
            actions
        }
    };

    actions.push(Action::SkipRender);

    actions
}

fn get_mode_after_command(mode_before: &Option<Mode>) -> Mode {
    if let Some(mode) = mode_before {
        match mode {
            Mode::Command(_) => unreachable!(),
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
        use yeet_keymap::message::Mode;

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
