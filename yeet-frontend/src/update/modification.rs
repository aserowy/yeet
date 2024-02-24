use yeet_keymap::message::{Buffer, Mode, PrintContent};

use crate::{action::Action, model::Model, update::current};

use super::{buffer, commandline, preview};

pub fn buffer(model: &mut Model, msg: &Buffer) -> Option<Vec<Action>> {
    // TODO: refactor into buffer mod
    match msg {
        Buffer::ChangeMode(from, to) => {
            if from == to {
                return None;
            }

            model.mode = to.clone();
            model.mode_before = Some(from.clone());

            let mut actions = vec![Action::ModeChanged];
            actions.extend(match from {
                Mode::Command => {
                    buffer::unfocus_buffer(&mut model.commandline.buffer);
                    commandline::update(model, Some(msg))
                }
                Mode::Insert | Mode::Navigation | Mode::Normal => {
                    buffer::unfocus_buffer(&mut model.current.buffer);
                    vec![]
                }
            });

            let content = format!("--{}--", to.to_string().to_uppercase());
            commandline::print(model, &[PrintContent::Info(content)]);

            actions.extend(match to {
                Mode::Command => {
                    buffer::focus_buffer(&mut model.commandline.buffer);
                    commandline::update(model, Some(msg))
                }
                Mode::Insert => {
                    buffer::focus_buffer(&mut model.current.buffer);
                    current::update(model, Some(msg));
                    vec![]
                }
                Mode::Navigation => {
                    // TODO: handle file operations: show pending with gray, refresh on operation success
                    // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
                    buffer::focus_buffer(&mut model.current.buffer);
                    current::update(model, Some(msg));
                    preview::viewport(model);
                    current::save_changes(model)
                }
                Mode::Normal => {
                    buffer::focus_buffer(&mut model.current.buffer);
                    current::update(model, Some(msg));
                    preview::viewport(model);
                    vec![]
                }
            });

            Some(actions)
        }
        Buffer::Modification(_) => match model.mode {
            Mode::Command => Some(commandline::update(model, Some(msg))),
            Mode::Insert | Mode::Normal => {
                current::update(model, Some(msg));
                None
            }
            Mode::Navigation => None,
        },
        Buffer::MoveCursor(_, _) | Buffer::MoveViewPort(_) => match model.mode {
            Mode::Command => {
                commandline::update(model, Some(msg));

                None
            }
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                let mut actions = Vec::new();
                current::update(model, Some(msg));

                if let Some(preview_actions) = preview::path(model, true, true) {
                    actions.extend(preview_actions);
                    model.preview.buffer.lines.clear();
                    preview::viewport(model);
                }

                Some(actions)
            }
        },
        Buffer::SaveBuffer(_) => Some(current::save_changes(model)),
    }
}
