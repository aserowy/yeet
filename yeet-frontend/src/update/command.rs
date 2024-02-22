use yeet_keymap::message::{Buffer, Message, Mode};

use crate::{
    action::{Action, PostView, PreView},
    model::Model,
    task::Task,
};

use super::path;

pub fn execute(cmd: &str, model: &mut Model) -> Vec<Action> {
    let change_mode_message = Message::Buffer(Buffer::ChangeMode(
        model.mode.clone(),
        get_mode_after_command(&model.mode_before),
    ));
    let change_mode_action = Action::PostView(PostView::Task(Task::EmitMessages(vec![
        change_mode_message.clone(),
    ])));

    let mut actions = match cmd {
        "d!" => {
            if let Some(path) = path::get_selected_path(model) {
                vec![
                    Action::PostView(PostView::Task(Task::DeletePath(path.clone()))),
                    change_mode_action,
                ]
            } else {
                vec![]
            }
        }
        "e!" => vec![Action::PostView(PostView::Task(Task::EmitMessages(vec![
            Message::NavigateToPath(model.current.path.clone()),
            change_mode_message,
        ])))],
        "histopt" => vec![
            Action::PostView(PostView::Task(Task::OptimizeHistory)),
            change_mode_action,
        ],
        "q" => vec![Action::PostView(PostView::Task(Task::EmitMessages(vec![
            Message::Quit,
        ])))],
        "w" => vec![Action::PostView(PostView::Task(Task::EmitMessages(vec![
            Message::Buffer(Buffer::SaveBuffer(None)),
            change_mode_message,
        ])))],
        "wq" => vec![Action::PostView(PostView::Task(Task::EmitMessages(vec![
            Message::Buffer(Buffer::SaveBuffer(None)),
            Message::Quit,
        ])))],
        _ => vec![change_mode_action],
    };
    actions.push(Action::PreView(PreView::SkipRender));

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
