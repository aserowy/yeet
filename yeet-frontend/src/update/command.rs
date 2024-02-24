use yeet_keymap::message::{Buffer, Message, Mode, PrintContent};

use crate::{action::Action, model::Model, task::Task};

use super::path;

pub fn execute(cmd: &str, model: &mut Model) -> Vec<Action> {
    let change_mode_message = Message::Buffer(Buffer::ChangeMode(
        model.mode.clone(),
        get_mode_after_command(&model.mode_before),
    ));
    let change_mode_action = Action::Task(Task::EmitMessages(vec![change_mode_message.clone()]));

    let mut actions = match cmd {
        "d!" => {
            let mut actions = vec![change_mode_action];
            if let Some(path) = path::get_selected_path(model) {
                actions.push(Action::Task(Task::DeletePath(path.clone())));
            }
            actions
        }
        "e!" => vec![Action::Task(Task::EmitMessages(vec![
            change_mode_message,
            Message::NavigateToPath(model.current.path.clone()),
        ]))],
        "histopt" => vec![change_mode_action, Action::Task(Task::OptimizeHistory)],
        "q" => vec![Action::Task(Task::EmitMessages(vec![Message::Quit]))],
        "reg" => {
            let content = model
                .register
                .print()
                .iter()
                .map(|cntnt| PrintContent::Info(cntnt.to_string()))
                .collect();

            vec![Action::Task(Task::EmitMessages(vec![Message::Print(
                content,
            )]))]
        }
        "w" => vec![Action::Task(Task::EmitMessages(vec![
            change_mode_message,
            Message::Buffer(Buffer::SaveBuffer(None)),
        ]))],
        "wq" => vec![Action::Task(Task::EmitMessages(vec![
            Message::Buffer(Buffer::SaveBuffer(None)),
            Message::Quit,
        ]))],
        _ => vec![change_mode_action],
    };
    actions.push(Action::SkipRender);

    actions
}

fn get_mode_after_command(mode_before: &Option<Mode>) -> Mode {
    if let Some(mode) = mode_before {
        match mode {
            Mode::Command => unreachable!(),
            Mode::Insert | Mode::Normal => Mode::Normal,
            Mode::Navigation => Mode::Navigation,
        }
    } else {
        Mode::default()
    }
}

mod test {
    #[test]
    fn test_get_mode_after_command() {
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
