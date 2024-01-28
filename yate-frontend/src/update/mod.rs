use ratatui::prelude::Rect;
use yate_keymap::message::{Message, Mode};

use crate::{
    event::PostRenderAction,
    layout::AppLayout,
    model::{buffer::ViewPort, Model},
};

mod buffer;
mod commandline;
mod current;
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

            return Some(vec![PostRenderAction::ModeChanged(to.clone())]);
        }
        Message::ExecuteCommand => {
            if let Some(cmd) = model.commandline.lines.first() {
                // FIX: this implementation bricks and sucks a**
                // maybe enable multiple AppResult as return to enable command and changemode
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
            if let Some(target) = path::get_selected_path(model) {
                if !target.is_dir() {
                    return None;
                }

                model.current_path = target.clone();

                current::update(model, layout, message);

                history::set_cursor_index(
                    &model.current_path,
                    &model.history,
                    &mut model.current_directory,
                );

                parent::update(model, layout, message);
                preview::update(model, layout, message);

                model.history.add(target);
            }
        }
        Message::SelectParent => {
            if let Some(parent) = &model.current_path.parent() {
                model.current_path = parent.to_path_buf();

                current::update(model, layout, message);

                history::set_cursor_index(
                    &model.current_path,
                    &model.history,
                    &mut model.current_directory,
                );

                parent::update(model, layout, message);
                preview::update(model, layout, message);
            }
        }
        Message::Quit => return Some(vec![PostRenderAction::Quit]),
    }

    None
}

fn set_viewport_dimensions(vp: &mut ViewPort, rect: &Rect) {
    vp.height = usize::from(rect.height);
    vp.width = usize::from(rect.width);
}
