use yeet_buffer::{
    message::BufferMessage,
    model::{BufferLine, CommandMode, Mode, SearchDirection},
    update::{focus_buffer, unfocus_buffer, update_buffer},
};
use yeet_keymap::message::PrintContent;

use crate::{
    action::Action,
    model::{register::RegisterScope, Model},
    update::update_current,
};

use super::{
    commandline::print_in_commandline, register::get_macro_register, save::persist_path_changes,
    viewport::set_viewport_dimensions,
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

fn update_commandline_on_mode_change(model: &mut Model) -> Vec<Action> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;
    let viewport = &mut commandline.viewport;

    set_viewport_dimensions(viewport, &commandline.layout.buffer);

    let command_mode = match &model.mode {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => {
            let from_command = model
                .mode_before
                .as_ref()
                .is_some_and(|mode| mode.is_command());

            if from_command {
                update_buffer(
                    viewport,
                    &model.mode,
                    buffer,
                    &BufferMessage::SetContent(vec![]),
                );
            }
            return Vec::new();
        }
    };

    match command_mode {
        CommandMode::Command | CommandMode::Search(_) => {
            update_buffer(viewport, &model.mode, buffer, &BufferMessage::ResetCursor);

            let prefix = match &command_mode {
                CommandMode::Command => Some(":".to_string()),
                CommandMode::Search(SearchDirection::Up) => Some("?".to_string()),
                CommandMode::Search(SearchDirection::Down) => Some("/".to_string()),
                CommandMode::PrintMultiline => unreachable!(),
            };

            let bufferline = BufferLine {
                prefix,
                ..Default::default()
            };

            update_buffer(
                viewport,
                &model.mode,
                buffer,
                &BufferMessage::SetContent(vec![bufferline]),
            );
        }
        CommandMode::PrintMultiline => {}
    };

    Vec::new()
}

fn set_commandline_content_to_mode(model: &mut Model) {
    if let Some(RegisterScope::Macro(identifier)) = &get_macro_register(&model.register) {
        set_recording_in_commandline(model, *identifier);
    } else {
        set_mode_in_commandline(model);
    };
}

pub fn set_recording_in_commandline(model: &mut Model, identifier: char) -> Vec<Action> {
    let content = format!("recording @{}", identifier);
    print_in_commandline(model, &[PrintContent::Default(content)]);
    Vec::new()
}

pub fn set_mode_in_commandline(model: &mut Model) -> Vec<Action> {
    let content = format!("--{}--", model.mode.to_string().to_uppercase());
    print_in_commandline(model, &[PrintContent::Default(content)]);
    Vec::new()
}
