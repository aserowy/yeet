use ratatui::prelude::Rect;
use yate_keymap::message::{Message, Mode};

use crate::{
    event::PostRenderAction,
    layout::AppLayout,
    model::{
        buffer::{viewport::ViewPort, BufferLine},
        Model,
    },
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
                }
                preview::update(model, layout, &Message::Refresh);

                Some(actions)
            }
        },
        Message::PathAdded(path) => {
            let mut buffer = vec![
                (
                    model.current.path.as_path(),
                    &mut model.current.buffer,
                    model.mode == Mode::Navigation,
                ),
                (
                    model.preview.path.as_path(),
                    &mut model.preview.buffer,
                    true,
                ),
            ];

            if let Some(parent) = path.parent() {
                buffer.push((parent, &mut model.parent.buffer, true));
            }

            if let Some(parent) = path.parent() {
                if let Some((_, buffer, sort)) = buffer.into_iter().find(|(p, _, _)| p == &parent) {
                    // TODO: better closer warmer: remove virtual entries... instead of this shiat
                    if let Some(basename) = path.file_name().map(|oss| oss.to_str()).flatten() {
                        let exists = buffer
                            .lines
                            .iter()
                            .find(|bl| bl.content == basename)
                            .is_some();

                        if !exists {
                            buffer.lines.push(path::get_bufferline_by_path(path));

                            if path.is_dir() {
                                let basepath = format!("{basename}/");

                                // NOTE: this removes virtual adds like 'dirname/filename'
                                let index = buffer
                                    .lines
                                    .iter()
                                    .enumerate()
                                    .find(|(_, bl)| bl.content.starts_with(&basepath))
                                    .map(|(i, _)| i);

                                if let Some(index) = index {
                                    buffer.lines.remove(index);
                                }
                            }
                        }

                        if sort {
                            directory::sort_content(&model.mode, buffer);
                        }

                        buffer::cursor::validate(&model.mode, buffer);
                        // TODO: correct cursor to stay on selection
                    }
                }
            }

            None
        }
        Message::PathRemoved(path) => {
            let mut buffer = vec![
                (model.current.path.as_path(), &mut model.current.buffer),
                (model.preview.path.as_path(), &mut model.preview.buffer),
            ];

            if let Some(parent) = path.parent() {
                buffer.push((parent, &mut model.parent.buffer));
            }

            if let Some(parent) = path.parent() {
                if let Some((_, buffer)) = buffer.into_iter().find(|(p, _)| p == &parent) {
                    // TODO: better closer warmer: remove virtual entries... instead of this shiat
                    if let Some(basename) = path.file_name().map(|oss| oss.to_str()).flatten() {
                        let index = buffer
                            .lines
                            .iter()
                            .enumerate()
                            .find(|(_, bl)| bl.content == basename)
                            .map(|(i, _)| i);

                        if let Some(index) = index {
                            buffer.lines.remove(index);
                            buffer::cursor::validate(&model.mode, buffer);
                        }
                    }
                }
            }

            None
        }
        Message::Refresh => {
            // TODO: handle undo state
            let mut actions = Vec::new();
            current::update(model, layout, message);
            current::set_content(model);

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

                parent::update(model, layout, message);
                preview::update(model, layout, message);

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

                parent::update(model, layout, message);
                preview::update(model, layout, message);

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

            let lines = match path::get_directory_content(&model.current.path) {
                Ok(content) => content,
                Err(_) => {
                    vec![BufferLine {
                        content: "Error reading directory".to_string(),
                        ..Default::default()
                    }]
                }
            };

            buffer::set_content(&model.mode, &mut model.current.buffer, lines);
            directory::sort_content(&model.mode, &mut model.current.buffer);

            current::update(model, layout, message);

            history::set_cursor_index(
                &model.current.path,
                &model.history,
                &mut model.current.buffer,
            );

            if let Some(preview) = path::set_preview_to_selected(model, true, true) {
                actions.extend(preview);
            }

            parent::update(model, layout, message);
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
