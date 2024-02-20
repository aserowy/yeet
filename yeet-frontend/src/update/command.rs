use yeet_keymap::message::{Buffer, Message, Mode};

use crate::{
    action::{Action, PostView},
    layout::AppLayout,
    model::Model,
    settings::Settings,
    task::Task,
};

use super::path;

pub fn execute(
    cmd: &str,
    settings: &Settings,
    model: &mut Model,
    layout: &AppLayout,
) -> Option<Vec<Action>> {
    let post_render_actions = match cmd {
        "d!" => path::get_selected_path(model).map(|path| {
            vec![Action::PostView(PostView::Task(Task::DeletePath(
                path.clone(),
            )))]
        }),
        "e!" => super::update(
            settings,
            model,
            layout,
            &Message::NavigateToPath(model.current.path.clone()),
        ),
        "histopt" => Some(vec![Action::PostView(PostView::Task(
            Task::OptimizeHistory,
        ))]),
        "q" => super::update(settings, model, layout, &Message::Quit),
        "w" => super::update(
            settings,
            model,
            layout,
            &Message::Buffer(Buffer::SaveBuffer(None)),
        ),
        "wq" => {
            let actions: Vec<_> = super::update(
                settings,
                model,
                layout,
                &Message::Buffer(Buffer::SaveBuffer(None)),
            )
            .into_iter()
            .flatten()
            .chain(
                super::update(settings, model, layout, &Message::Quit)
                    .into_iter()
                    .flatten(),
            )
            .collect();

            if actions.is_empty() {
                None
            } else {
                Some(actions)
            }
        }
        _ => None,
    };

    let mode_changed_actions = super::update(
        settings,
        model,
        layout,
        &Message::Buffer(Buffer::ChangeMode(
            model.mode.clone(),
            get_mode_after_command(&model.mode_before),
        )),
    );

    Some(
        post_render_actions
            .into_iter()
            .flatten()
            .chain(mode_changed_actions.into_iter().flatten())
            .collect(),
    )
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
