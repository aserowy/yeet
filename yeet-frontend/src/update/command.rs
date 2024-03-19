use yeet_keymap::message::{Buffer, Message, Mode, PrintContent};

use crate::{
    action::Action,
    model::Model,
    task::Task,
    update::{mark, qfix},
};

#[tracing::instrument(skip(model))]
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

    tracing::debug!("executing command: {:?}", cmd);

    // NOTE: all file commands like e.g. d! should use preview path as target to enable cdo
    let mut actions = match cmd {
        ("cdo", command) => {
            let mut commands = Vec::new();
            for path in &model.qfix.entries {
                commands.push(Message::NavigateToPathAsPreview(path.clone()));
                commands.push(Message::ExecuteCommandString(command.to_owned()));
            }
            commands.reverse();

            tracing::debug!("cdo commands set: {:?}", commands);
            model.qfix.do_command_stack = Some(commands);

            vec![change_mode_action]
        }
        ("cfirst", "") => {
            model.qfix.current_index = 0;

            let path = match model.qfix.entries.first() {
                Some(it) => it,
                None => return vec![change_mode_action],
            };

            vec![
                change_mode_action,
                Action::EmitMessages(vec![Message::NavigateToPathAsPreview(path.clone())]),
            ]
        }
        ("cl", "") => {
            let content = qfix::print(&model.qfix)
                .iter()
                .enumerate()
                .map(|(i, cntnt)| {
                    if i == model.qfix.current_index + 1 {
                        PrintContent::Information(cntnt.to_string())
                    } else {
                        PrintContent::Default(cntnt.to_string())
                    }
                })
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
                        Action::EmitMessages(vec![Message::NavigateToPathAsPreview(it.clone())]),
                    ]
                }
                None => {
                    vec![
                        Action::SkipRender,
                        Action::EmitMessages(vec![Message::ExecuteCommandString(
                            "cfirst".to_string(),
                        )]),
                    ]
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
                        Action::EmitMessages(vec![Message::NavigateToPathAsPreview(it.clone())]),
                    ]
                }
                None => {
                    vec![
                        Action::SkipRender,
                        Action::EmitMessages(vec![Message::ExecuteCommandString("cN".to_string())]),
                    ]
                }
            }
        }
        ("d!", "") => {
            let mut actions = vec![change_mode_action];
            if let Some(path) = &model.preview.path {
                tracing::info!("deleting path: {:?}", path);
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
        ("e!", "") => {
            let navigation = if let Some(path) = &model.preview.path {
                Message::NavigateToPathAsPreview(path.to_path_buf())
            } else {
                Message::NavigateToPath(model.current.path.clone())
            };

            vec![Action::EmitMessages(vec![change_mode_message, navigation])]
        }
        ("histopt", "") => vec![change_mode_action, Action::Task(Task::OptimizeHistory)],
        ("jnk", "") => {
            let content = model
                .junk
                .print()
                .iter()
                .map(|cntnt| PrintContent::Default(cntnt.to_string()))
                .collect();

            vec![Action::EmitMessages(vec![Message::Print(content)])]
        }
        ("marks", "") => {
            let content = mark::print(&model.marks)
                .iter()
                .map(|cntnt| PrintContent::Default(cntnt.to_string()))
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
