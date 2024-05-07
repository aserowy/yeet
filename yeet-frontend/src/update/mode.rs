use yeet_buffer::{
    message::BufferMessage,
    model::Mode,
    update::{focus_buffer, unfocus_buffer},
};

use crate::{action::Action, model::Model, update::buffer::update_current};

use super::{
    commandline::{set_commandline_content_to_mode, update_commandline_on_mode_change},
    save::persist_path_changes,
};

pub fn change_mode(model: &mut Model, from: &Mode, to: &Mode) -> Vec<Action> {
    match (from, to) {
        (Mode::Command(_), Mode::Command(_))
        | (Mode::Insert, Mode::Insert)
        | (Mode::Navigation, Mode::Navigation)
        | (Mode::Normal, Mode::Normal) => return Vec::new(),
        _ => {}
    }

    model.mode = to.clone();
    model.mode_before = Some(from.clone());

    let mut actions = vec![Action::ModeChanged];
    actions.extend(match from {
        Mode::Command(_) => {
            unfocus_buffer(&mut model.commandline.buffer);
            update_commandline_on_mode_change(model)
        }
        Mode::Insert | Mode::Navigation | Mode::Normal => {
            unfocus_buffer(&mut model.files.current.buffer);
            vec![]
        }
    });

    set_commandline_content_to_mode(model);

    let msg = BufferMessage::ChangeMode(from.clone(), to.clone());
    actions.extend(match to {
        Mode::Command(_) => {
            focus_buffer(&mut model.commandline.buffer);
            update_commandline_on_mode_change(model)
        }
        Mode::Insert => {
            focus_buffer(&mut model.files.current.buffer);
            update_current(model, &msg);
            vec![]
        }
        Mode::Navigation => {
            // TODO: handle file operations: show pending with gray, refresh on operation success
            // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
            focus_buffer(&mut model.files.current.buffer);
            update_current(model, &msg);
            persist_path_changes(model)
        }
        Mode::Normal => {
            focus_buffer(&mut model.files.current.buffer);
            update_current(model, &msg);
            vec![]
        }
    });

    actions
}
