use yate_keymap::message::{Message, Mode};

use crate::{event::AppResult, layout::AppLayout, model::Model};

mod buffer;
mod commandline;
mod current;
mod parent;
mod path;
mod preview;

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) -> Option<AppResult> {
    match message {
        Message::ChangeKeySequence(sequence) => {
            model.key_sequence = sequence.clone();
        }
        Message::ChangeMode(from, to) => {
            if from == to {
                return None;
            }

            model.mode = to.clone();

            match from {
                Mode::Command => {
                    buffer::unfocus_buffer(&mut model.commandline);
                    commandline::update(model, layout, message);
                }
                Mode::Normal => {
                    buffer::unfocus_buffer(&mut model.current_directory);
                }
            }

            match to {
                Mode::Command => {
                    buffer::focus_buffer(&mut model.commandline);
                    commandline::update(model, layout, message);
                }
                Mode::Normal => {
                    // NOTE: add file modification handling
                    buffer::focus_buffer(&mut model.current_directory);
                }
            }

            return Some(AppResult::ModeChanged(to.clone()));
        }
        Message::ExecuteCommand => {
            if let Some(cmd) = model.commandline.lines.first() {
                match cmd.content.as_str() {
                    "q" => return update(model, layout, &Message::Quit),
                    _ => {
                        // TODO: add notification in cmd line?
                        return update(
                            model,
                            layout,
                            &Message::ChangeMode(model.mode.clone(), Mode::Normal),
                        );
                    }
                }
            }
        }
        Message::Modification(_) => match model.mode {
            Mode::Normal => {
                // NOTE: add file modification handling
                current::update(model, layout, message);
            }
            Mode::Command => {
                commandline::update(model, layout, message);
            }
        },
        Message::MoveCursor(_, _) => match model.mode {
            Mode::Normal => {
                current::update(model, layout, message);
                preview::update(model, layout, message);
            }
            Mode::Command => {
                commandline::update(model, layout, message);
            }
        },
        Message::MoveViewPort(_) => match model.mode {
            Mode::Normal => {
                current::update(model, layout, message);
                preview::update(model, layout, message);
            }
            Mode::Command => {
                commandline::update(model, layout, message);
            }
        },
        Message::Refresh => {
            commandline::update(model, layout, message);
            current::update(model, layout, message);
            parent::update(model, layout, message);
            preview::update(model, layout, message);
        }
        Message::SelectCurrent => {
            if let Some(target) = path::get_target_path(model) {
                if !target.is_dir() {
                    return None;
                }

                model.current_path = target;

                current::update(model, layout, message);
                parent::update(model, layout, message);
                preview::update(model, layout, message);
            }
        }
        Message::SelectParent => {
            if let Some(parent) = &model.current_path.parent() {
                model.current_path = parent.to_path_buf();

                current::update(model, layout, message);
                parent::update(model, layout, message);
                preview::update(model, layout, message);
            }
        }
        Message::Quit => return Some(AppResult::Quit),
    }

    None
}
