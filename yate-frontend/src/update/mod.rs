use ratatui::prelude::Rect;
use yate_keymap::message::{Buffer, Message, Mode};

use crate::{
    event::{PostRenderAction, RenderAction},
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

// TODO: refactor into right abstraction level (what ever right means here :D)
pub fn update(
    model: &mut Model,
    layout: &AppLayout,
    message: &Message,
) -> Option<Vec<RenderAction>> {
    match message {
        Message::Buffer(msg) => {
            match msg {
                Buffer::ChangeMode(from, to) => {
                    if from == to {
                        return None;
                    }

                    model.mode = to.clone();
                    model.mode_before = Some(from.clone());

                    match from {
                        Mode::Command => {
                            buffer::unfocus_buffer(&mut model.commandline);
                            commandline::update(model, layout, msg);
                        }
                        Mode::Insert | Mode::Navigation | Mode::Normal => {
                            buffer::unfocus_buffer(&mut model.current.buffer);
                        }
                    }

                    let post_render_actions = match to {
                        Mode::Command => {
                            buffer::focus_buffer(&mut model.commandline);
                            commandline::update(model, layout, msg);

                            None
                        }
                        Mode::Insert => {
                            buffer::focus_buffer(&mut model.current.buffer);
                            current::update(model, layout, Some(msg));

                            None
                        }
                        Mode::Navigation => {
                            // TODO: handle file operations: show pending with gray, refresh on operation success
                            // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
                            buffer::focus_buffer(&mut model.current.buffer);
                            current::update(model, layout, Some(msg));
                            preview::update(model, layout, None);

                            current::save_changes(model)
                        }
                        Mode::Normal => {
                            buffer::focus_buffer(&mut model.current.buffer);
                            current::update(model, layout, Some(msg));
                            preview::update(model, layout, None);

                            None
                        }
                    };

                    if let Some(mut post_render_action_vec) = post_render_actions {
                        post_render_action_vec.push(RenderAction::Post(
                            PostRenderAction::ModeChanged(to.clone()),
                        ));

                        Some(post_render_action_vec)
                    } else {
                        Some(vec![RenderAction::Post(PostRenderAction::ModeChanged(
                            to.clone(),
                        ))])
                    }
                }
                Buffer::Modification(_) => {
                    match model.mode {
                        Mode::Command => commandline::update(model, layout, msg),
                        Mode::Insert | Mode::Normal => current::update(model, layout, Some(msg)),
                        Mode::Navigation => {}
                    }

                    None
                }
                Buffer::MoveCursor(_, _) | Buffer::MoveViewPort(_) => match model.mode {
                    Mode::Command => {
                        commandline::update(model, layout, msg);

                        None
                    }
                    Mode::Insert | Mode::Navigation | Mode::Normal => {
                        let mut actions = Vec::new();
                        current::update(model, layout, Some(msg));

                        if let Some(preview_actions) =
                            path::set_preview_to_selected(model, true, true)
                        {
                            actions.extend(preview_actions);
                            model.preview.buffer.lines.clear();
                            preview::update(model, layout, None);
                        }

                        Some(actions)
                    }
                },
                Buffer::SaveBuffer(_) => current::save_changes(model),
            }
        }
        Message::ChangeKeySequence(sequence) => {
            model.key_sequence = sequence.clone();
            None
        }
        Message::ExecuteCommand => {
            if let Some(cmd) = model.commandline.lines.first() {
                update(
                    model,
                    layout,
                    &Message::ExecuteCommandString(cmd.content.clone()),
                )
            } else {
                None
            }
        }
        Message::ExecuteCommandString(command) => {
            let post_render_actions = match command.as_str() {
                "e!" => update(
                    model,
                    layout,
                    &Message::SelectPath(model.current.path.clone()),
                ),
                "histopt" => Some(vec![RenderAction::Post(PostRenderAction::Task(
                    Task::OptimizeHistory,
                ))]),
                "q" => update(model, layout, &Message::Quit),
                "w" => update(model, layout, &Message::Buffer(Buffer::SaveBuffer(None))),
                "wq" => {
                    let actions: Vec<_> =
                        update(model, layout, &Message::Buffer(Buffer::SaveBuffer(None)))
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
        Message::PathEnumerationContentChanged(path, contents) => {
            // TODO: handle unsaved changes
            let mut buffer = vec![
                (model.current.path.as_path(), &mut model.current.buffer),
                (model.preview.path.as_path(), &mut model.preview.buffer),
            ];

            if let Some(parent) = &model.parent.path {
                buffer.push((parent, &mut model.parent.buffer));
            }

            let mut actions = Vec::new();
            if let Some((_, buffer)) = buffer.into_iter().find(|(p, _)| p == path) {
                let content = contents
                    .iter()
                    .map(|(knd, cntnt)| path::get_bufferline_by_enumeration_content(knd, cntnt))
                    .collect();

                buffer::set_content(&model.mode, buffer, content);

                if let Some(preview_actions) = path::set_preview_to_selected(model, true, true) {
                    actions.extend(preview_actions);
                    model.preview.buffer.lines.clear();
                    preview::update(model, layout, None);
                }
            }

            if actions.is_empty() {
                None
            } else {
                Some(actions)
            }
        }
        Message::PathEnumerationFinished(path) => {
            if model.mode != Mode::Navigation {
                return None;
            }

            let mut buffer = vec![
                (model.current.path.as_path(), &mut model.current.buffer),
                (model.preview.path.as_path(), &mut model.preview.buffer),
            ];

            if let Some(parent) = &model.parent.path {
                buffer.push((parent, &mut model.parent.buffer));
            }

            let mut actions = Vec::new();
            if let Some((path, buffer)) = buffer.into_iter().find(|(p, _)| p == path) {
                directory::sort_content(&model.mode, buffer);
                history::set_cursor_index(path, &model.history, buffer);

                if let Some(preview_actions) = path::set_preview_to_selected(model, true, true) {
                    actions.extend(preview_actions);
                    model.preview.buffer.lines.clear();
                    preview::update(model, layout, None);
                }
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
                preview::update(model, layout, None);
            }

            if actions.is_empty() {
                None
            } else {
                Some(actions)
            }
        }
        Message::PathsAdded(paths) => {
            directory::add_paths(model, paths);

            let mut actions = Vec::new();
            if let Some(preview_actions) = path::set_preview_to_selected(model, true, true) {
                actions.extend(preview_actions);
                model.preview.buffer.lines.clear();
                preview::update(model, layout, None);
            }

            if actions.is_empty() {
                None
            } else {
                Some(actions)
            }
        }
        Message::PreviewLoaded(path, content) => {
            if path == &model.preview.path {
                let content = content
                    .iter()
                    .map(|s| BufferLine {
                        content: s.to_string(),
                        ..Default::default()
                    })
                    .collect();

                buffer::set_content(&model.mode, &mut model.preview.buffer, content);
                preview::update(model, layout, None);
            }

            None
        }
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
                current::update(model, layout, None);

                history::set_cursor_index(
                    &model.current.path,
                    &model.history,
                    &mut model.current.buffer,
                );

                if let Some(preview_actions) = path::set_preview_to_selected(model, false, true) {
                    actions.extend(preview_actions);
                }
                model.preview.buffer.lines.clear();
                preview::update(model, layout, None);

                buffer::set_content(&model.mode, &mut model.parent.buffer, current_content);
                parent::update(model, layout, None);

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
                current::update(model, layout, None);

                history::set_cursor_index(
                    &model.current.path,
                    &model.history,
                    &mut model.current.buffer,
                );

                if let Some(preview_actions) = path::set_preview_to_selected(model, true, false) {
                    actions.extend(preview_actions);
                }
                buffer::set_content(&model.mode, &mut model.preview.buffer, current_content);
                preview::update(model, layout, None);

                model.parent.buffer.lines.clear();
                parent::update(model, layout, None);

                Some(actions)
            } else {
                None
            }
        }
        Message::SelectPath(path) => {
            let directory = if path.is_file() {
                path.parent().unwrap()
            } else {
                path.as_path()
            };

            let mut actions = Vec::new();
            if let Some(current_actions) = path::set_current_to_path(model, directory) {
                actions.extend(current_actions);
            }

            model.current.buffer.lines.clear();
            current::update(model, layout, None);

            history::set_cursor_index(
                &model.current.path,
                &model.history,
                &mut model.current.buffer,
            );

            if let Some(preview) = path::set_preview_to_selected(model, true, true) {
                actions.extend(preview);
            }

            model.parent.buffer.lines.clear();
            parent::update(model, layout, None);

            model.preview.buffer.lines.clear();
            preview::update(model, layout, None);

            model.history.add(&model.current.path);

            Some(actions)
        }
        Message::Quit => Some(vec![
            RenderAction::Post(PostRenderAction::Task(Task::SaveHistory(
                model.history.clone(),
            ))),
            RenderAction::Post(PostRenderAction::Quit),
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
