use ratatui::prelude::Rect;
use yeet_keymap::message::{Buffer, Message, Mode};

use crate::{
    action::{Action, PostView, PreView},
    model::{
        buffer::{viewport::ViewPort, BufferLine},
        Model,
    },
    settings::Settings,
    task::Task,
};

mod buffer;
mod command;
pub mod commandline;
mod current;
mod directory;
mod history;
mod parent;
mod path;
mod preview;

// TODO: refactor into right abstraction level (what ever right means here :D)
pub fn update(settings: &Settings, model: &mut Model, message: &Message) -> Option<Vec<Action>> {
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
                            buffer::unfocus_buffer(&mut model.commandline.buffer);
                            commandline::update(model, Some(msg));
                        }
                        Mode::Insert | Mode::Navigation | Mode::Normal => {
                            buffer::unfocus_buffer(&mut model.current.buffer);
                        }
                    }

                    let post_render_actions = match to {
                        Mode::Command => {
                            buffer::focus_buffer(&mut model.commandline.buffer);
                            commandline::update(model, Some(msg));

                            None
                        }
                        Mode::Insert => {
                            buffer::focus_buffer(&mut model.current.buffer);
                            commandline::update(model, Some(msg));
                            current::update(model, Some(msg));

                            None
                        }
                        Mode::Navigation => {
                            // TODO: handle file operations: show pending with gray, refresh on operation success
                            // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
                            buffer::focus_buffer(&mut model.current.buffer);
                            commandline::update(model, Some(msg));
                            current::update(model, Some(msg));
                            preview::update(model, None);

                            current::save_changes(model)
                        }
                        Mode::Normal => {
                            buffer::focus_buffer(&mut model.current.buffer);
                            commandline::update(model, Some(msg));
                            current::update(model, Some(msg));
                            preview::update(model, None);

                            None
                        }
                    };

                    if let Some(mut post_render_action_vec) = post_render_actions {
                        post_render_action_vec
                            .push(Action::PostView(PostView::ModeChanged(to.clone())));

                        Some(post_render_action_vec)
                    } else {
                        Some(vec![Action::PostView(PostView::ModeChanged(to.clone()))])
                    }
                }
                Buffer::Modification(_) => {
                    match model.mode {
                        Mode::Command => commandline::update(model, Some(msg)),
                        Mode::Insert | Mode::Normal => current::update(model, Some(msg)),
                        Mode::Navigation => {}
                    }

                    None
                }
                Buffer::MoveCursor(_, _) | Buffer::MoveViewPort(_) => match model.mode {
                    Mode::Command => {
                        commandline::update(model, Some(msg));

                        None
                    }
                    Mode::Insert | Mode::Navigation | Mode::Normal => {
                        let mut actions = Vec::new();
                        current::update(model, Some(msg));

                        if let Some(preview_actions) =
                            path::set_preview_to_selected(model, true, true)
                        {
                            actions.extend(preview_actions);
                            model.preview.buffer.lines.clear();
                            preview::update(model, None);
                        }

                        Some(actions)
                    }
                },
                Buffer::SaveBuffer(_) => current::save_changes(model),
            }
        }
        Message::ExecuteCommand => {
            let action = if let Some(cmd) = model.commandline.buffer.lines.last() {
                Message::ExecuteCommandString(cmd.content.clone())
            } else {
                Message::Buffer(Buffer::ChangeMode(model.mode.clone(), Mode::Navigation))
            };

            Some(vec![Action::PostView(PostView::Task(Task::EmitMessages(
                vec![action],
            )))])
        }
        Message::ExecuteCommandString(command) => Some(command::execute(command, model)),
        Message::KeySequenceChanged(sequence) => {
            model.key_sequence = sequence.clone();
            commandline::update(model, None);
            None
        }
        Message::NavigateToParent => {
            if model.mode != Mode::Navigation {
                None
            } else if let Some(mut actions) = path::set_current_to_parent(model) {
                let current_content = model.current.buffer.lines.clone();

                buffer::set_content(
                    &model.mode,
                    &mut model.current.buffer,
                    model.parent.buffer.lines.clone(),
                );
                current::update(model, None);

                history::set_cursor_index(
                    &model.current.path,
                    &model.history,
                    &mut model.current.buffer,
                );

                if let Some(preview_actions) = path::set_preview_to_selected(model, true, false) {
                    actions.extend(preview_actions);
                }
                buffer::set_content(&model.mode, &mut model.preview.buffer, current_content);
                preview::update(model, None);

                model.parent.buffer.lines.clear();
                parent::update(model, None);

                Some(actions)
            } else {
                None
            }
        }
        Message::NavigateToPath(path) => {
            // TODO: check in set current to path and extend enumeration request with filename
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
            current::update(model, None);

            model.parent.buffer.lines.clear();
            parent::update(model, None);

            model.preview.buffer.lines.clear();
            preview::update(model, None);

            model.history.add(&model.current.path);

            Some(actions)
        }
        Message::NavigateToSelected => {
            if model.mode != Mode::Navigation {
                None
            } else if let Some(mut actions) = path::set_current_to_selected(model) {
                let current_content = model.current.buffer.lines.clone();

                buffer::set_content(
                    &model.mode,
                    &mut model.current.buffer,
                    model.preview.buffer.lines.clone(),
                );
                current::update(model, None);

                history::set_cursor_index(
                    &model.current.path,
                    &model.history,
                    &mut model.current.buffer,
                );

                if let Some(preview_actions) = path::set_preview_to_selected(model, false, true) {
                    actions.extend(preview_actions);
                }
                model.preview.buffer.lines.clear();
                preview::update(model, None);

                buffer::set_content(&model.mode, &mut model.parent.buffer, current_content);
                parent::update(model, None);

                model.history.add(&model.current.path);

                Some(actions)
            } else {
                None
            }
        }
        Message::OpenSelected => {
            if model.mode != Mode::Navigation {
                return None;
            }

            if let Some(selected) = path::get_selected_path(model) {
                if settings.stdout_on_open {
                    Some(vec![Action::PostView(PostView::Quit(Some(
                        selected.to_string_lossy().to_string(),
                    )))])
                } else {
                    Some(vec![Action::PreView(PreView::Open(selected))])
                }
            } else {
                None
            }
        }
        Message::PasteRegister(register) => {
            if let Some(entry) = model.register.get(register) {
                Some(vec![Action::PostView(PostView::Task(Task::RestorePath(
                    entry,
                    model.current.path.clone(),
                )))])
            } else {
                None
            }
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
                    preview::update(model, None);
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
                    preview::update(model, None);
                }
            }

            if actions.is_empty() {
                None
            } else {
                Some(actions)
            }
        }
        Message::PathRemoved(path) => {
            if path.starts_with(&model.register.path) {
                model.register.remove(path);
                None
            } else {
                directory::remove_path(model, path);

                let mut actions = Vec::new();
                if let Some(preview_actions) = path::set_preview_to_selected(model, true, true) {
                    actions.extend(preview_actions);
                    model.preview.buffer.lines.clear();
                    preview::update(model, None);
                }

                if actions.is_empty() {
                    None
                } else {
                    Some(actions)
                }
            }
        }
        Message::PathsAdded(paths) => {
            directory::add_paths(model, paths);

            let mut actions = Vec::new();
            if let Some(preview_actions) = path::set_preview_to_selected(model, true, true) {
                actions.extend(preview_actions);
                model.preview.buffer.lines.clear();
                preview::update(model, None);
            }

            if actions.is_empty() {
                None
            } else {
                Some(actions)
            }
        }
        Message::PathsWriteFinished(paths) => {
            let mut actions = vec![Action::PreView(PreView::SkipRender)];
            for path in paths {
                if path.starts_with(&model.register.path) {
                    if let Some(old_entry) = model.register.add_or_update(path) {
                        actions.push(Action::PostView(PostView::Task(Task::DeleteRegisterEntry(
                            old_entry,
                        ))));
                    }
                }
            }

            Some(actions)
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
                preview::update(model, None);
            }

            None
        }
        Message::Print(content) => commandline::print(model, content),
        Message::Resize(x, y) => Some(vec![Action::PreView(PreView::Resize(*x, *y))]),
        Message::Quit => Some(vec![Action::PostView(PostView::Quit(None))]),
        Message::YankSelected => {
            if let Some(selected) = path::get_selected_path(model) {
                let mut tasks = Vec::new();

                let (entry, old_entry) = model.register.yank(&selected);
                tasks.push(Action::PostView(PostView::Task(Task::YankPath(entry))));
                if let Some(old_entry) = old_entry {
                    tasks.push(Action::PostView(PostView::Task(Task::DeleteRegisterEntry(
                        old_entry,
                    ))));
                }

                Some(tasks)
            } else {
                None
            }
        }
    }
}

fn set_viewport_dimensions(vp: &mut ViewPort, rect: &Rect) {
    vp.height = usize::from(rect.height);
    vp.width = usize::from(rect.width);
}
