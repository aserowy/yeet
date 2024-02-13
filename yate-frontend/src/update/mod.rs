use ratatui::prelude::Rect;
use yate_keymap::message::{Message, Mode};

use crate::{
    event::PostRenderAction,
    layout::AppLayout,
    model::{buffer::viewport::ViewPort, Model},
    task::Task,
};

mod buffer;
mod commandline;
mod current;
mod directory;
mod history;
mod parent;
mod path;
mod preview;

pub fn update(
    model: &mut Model,
    layout: &AppLayout,
    message: &Message,
) -> Option<Vec<PostRenderAction>> {
    match message {
        Message::ChangeKeySequence(sequence) => {
            model.key_sequence = sequence.clone();
            None
        }
        Message::ChangeMode(from, to) => {
            if from == to {
                return None;
            }

            model.mode = to.clone();
            model.mode_before = Some(from.clone());

            match from {
                Mode::Command => {
                    buffer::unfocus_buffer(&mut model.commandline);
                    commandline::update(model, layout, message);
                }
                Mode::Insert | Mode::Navigation | Mode::Normal => {
                    buffer::unfocus_buffer(&mut model.current.buffer);
                }
            }

            let post_render_actions = match to {
                Mode::Command => {
                    buffer::focus_buffer(&mut model.commandline);
                    commandline::update(model, layout, message);

                    None
                }
                Mode::Insert => {
                    buffer::focus_buffer(&mut model.current.buffer);
                    current::update(model, layout, message);

                    None
                }
                Mode::Navigation => {
                    // TODO: handle file operations: show pending with gray, refresh on operation success
                    // - depending on info in notify message, replace exact line or refresh all
                    buffer::focus_buffer(&mut model.current.buffer);
                    current::update(model, layout, message);
                    preview::update(model, layout, &Message::Refresh);

                    current::save_changes(model)
                }
                Mode::Normal => {
                    buffer::focus_buffer(&mut model.current.buffer);
                    current::update(model, layout, message);
                    preview::update(model, layout, &Message::Refresh);

                    None
                }
            };

            if let Some(mut post_render_action_vec) = post_render_actions {
                post_render_action_vec.push(PostRenderAction::ModeChanged(to.clone()));

                Some(post_render_action_vec)
            } else {
                Some(vec![PostRenderAction::ModeChanged(to.clone())])
            }
        }
        Message::ExecuteCommand => {
            if let Some(cmd) = model.commandline.lines.first() {
                let post_render_actions = match cmd.content.as_str() {
                    "e!" => update(model, layout, &Message::Refresh),
                    "histopt" => Some(vec![PostRenderAction::Task(Task::OptimizeHistory)]),
                    "q" => update(model, layout, &Message::Quit),
                    "w" => update(model, layout, &Message::SaveBuffer(None)),
                    "wq" => {
                        let actions: Vec<_> = update(model, layout, &Message::SaveBuffer(None))
                            .into_iter()
                            .flatten()
                            .chain(update(model, layout, &Message::Quit).into_iter().flatten())
                            .collect();

                        if actions.is_empty() {
                            None
                        } else {
                            Some(actions)
                        }
                    }
                    _ => None,
                };

                let mode_changed_actions = update(
                    model,
                    layout,
                    &Message::ChangeMode(
                        model.mode.clone(),
                        get_mode_after_command(&model.mode_before),
                    ),
                );

                Some(
                    post_render_actions
                        .into_iter()
                        .flatten()
                        .chain(mode_changed_actions.into_iter().flatten())
                        .collect(),
                )
            } else {
                None
            }
        }
        Message::Modification(_) => {
            match model.mode {
                Mode::Command => commandline::update(model, layout, message),
                Mode::Insert | Mode::Normal => current::update(model, layout, message),
                Mode::Navigation => {}
            }

            None
        }
        Message::MoveCursor(_, _) | Message::MoveViewPort(_) => match model.mode {
            Mode::Command => {
                commandline::update(model, layout, message);

                None
            }
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                let mut actions = Vec::new();
                current::update(model, layout, message);

                if let Some(preview_actions) = path::set_preview_to_selected(model, true, true) {
                    actions.extend(preview_actions);
                    model.preview.buffer.lines.clear();
                    preview::update(model, layout, &Message::Refresh);
                }

                Some(actions)
            }
        },
        Message::PathsAdded(paths) => {
            directory::add_paths(model, paths);

            let mut actions = Vec::new();
            if let Some(preview_actions) = path::set_preview_to_selected(model, true, true) {
                actions.extend(preview_actions);
                model.preview.buffer.lines.clear();
                preview::update(model, layout, &Message::Refresh);
            }

            if actions.is_empty() {
                None
            } else {
                Some(actions)
            }
        }
        Message::PathRemoved(path) => {
            directory::remove_path(model, path);

            let mut actions = Vec::new();
            if let Some(preview_actions) = path::set_preview_to_selected(model, true, true) {
                actions.extend(preview_actions);
                model.preview.buffer.lines.clear();
                preview::update(model, layout, &Message::Refresh);
            }

            if actions.is_empty() {
                None
            } else {
                Some(actions)
            }
        }
        Message::Refresh => {
            // TODO: handle undo state
            // TODO: remove dir contents and restart enumeration
            let mut actions = Vec::new();
            current::update(model, layout, message);

            if let Some(preview_actions) = path::set_preview_to_selected(model, true, true) {
                actions.extend(preview_actions);
            }
            preview::update(model, layout, message);

            commandline::update(model, layout, message);
            parent::update(model, layout, message);

            Some(actions)
        }
        Message::SaveBuffer(_) => current::save_changes(model),
        Message::SelectCurrent => {
            if model.mode != Mode::Navigation {
                None
            } else if let Some(mut actions) = path::set_current_to_selected(model) {
                let current_content = model.current.buffer.lines.clone();

                buffer::set_content(
                    &model.mode,
                    &mut model.current.buffer,
                    model.preview.buffer.lines.clone(),
                );
                current::update(model, layout, message);

                history::set_cursor_index(
                    &model.current.path,
                    &model.history,
                    &mut model.current.buffer,
                );

                if let Some(preview_actions) = path::set_preview_to_selected(model, false, true) {
                    actions.extend(preview_actions);
                }
                model.preview.buffer.lines.clear();
                preview::update(model, layout, message);

                buffer::set_content(&model.mode, &mut model.parent.buffer, current_content);
                parent::update(model, layout, message);

                model.history.add(&model.current.path);

                Some(actions)
            } else {
                None
            }
        }
        Message::SelectParent => {
            if model.mode != Mode::Navigation {
                None
            } else if let Some(mut actions) = path::set_current_to_parent(model) {
                let current_content = model.current.buffer.lines.clone();

                buffer::set_content(
                    &model.mode,
                    &mut model.current.buffer,
                    model.parent.buffer.lines.clone(),
                );
                current::update(model, layout, message);

                history::set_cursor_index(
                    &model.current.path,
                    &model.history,
                    &mut model.current.buffer,
                );

                if let Some(preview_actions) = path::set_preview_to_selected(model, true, false) {
                    actions.extend(preview_actions);
                }
                buffer::set_content(&model.mode, &mut model.preview.buffer, current_content);
                preview::update(model, layout, message);

                model.parent.buffer.lines.clear();
                parent::update(model, layout, message);

                Some(actions)
            } else {
                None
            }
        }
        Message::SelectPath(path) => {
            let mut actions = Vec::new();
            if let Some(current_actions) = path::set_current_to_path(model, path) {
                actions.extend(current_actions);
            }

            model.current.buffer.lines.clear();
            current::update(model, layout, message);

            // TODO: add finished enumeration and set history and preview
            history::set_cursor_index(
                &model.current.path,
                &model.history,
                &mut model.current.buffer,
            );

            if let Some(preview) = path::set_preview_to_selected(model, true, true) {
                actions.extend(preview);
            }

            model.parent.buffer.lines.clear();
            parent::update(model, layout, message);

            model.preview.buffer.lines.clear();
            preview::update(model, layout, message);

            Some(actions)
        }
        Message::Quit => Some(vec![
            PostRenderAction::OptimizeHistory,
            PostRenderAction::Quit,
        ]),
    }
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

fn set_viewport_dimensions(vp: &mut ViewPort, rect: &Rect) {
    vp.height = usize::from(rect.height);
    vp.width = usize::from(rect.width);
}

mod test {
    #[test]
    fn test_get_mode_after_command() {
        use yate_keymap::message::Mode;

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
