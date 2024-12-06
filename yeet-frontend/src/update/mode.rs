use yeet_buffer::{
    message::BufferMessage,
    model::{BufferLine, CommandMode, Mode, SearchDirection},
    update::{focus_buffer, unfocus_buffer, update_buffer},
};
use yeet_keymap::message::PrintContent;

use crate::{
    action::Action,
    layout::AppLayout,
    model::{
        junkyard::JunkYard,
        register::{Register, RegisterScope},
        CommandLine, FileTreeBuffer, ModeState,
    },
    update::update_current,
};

use super::{
    commandline::print_in_commandline, register::get_macro_register, save::persist_path_changes,
    viewport::set_viewport_dimensions,
};

pub fn change_mode(
    commandline: &mut CommandLine,
    junk: &mut JunkYard,
    layout: &AppLayout,
    register: &Register,
    modes: &mut ModeState,
    buffer: &mut FileTreeBuffer,
    from: &Mode,
    to: &Mode,
) -> Vec<Action> {
    match (from, to) {
        (Mode::Command(_), Mode::Command(_))
        | (Mode::Insert, Mode::Insert)
        | (Mode::Navigation, Mode::Navigation)
        | (Mode::Normal, Mode::Normal) => return Vec::new(),
        _ => {}
    }

    modes.current = to.clone();
    modes.previous = Some(from.clone());

    let mut actions = vec![Action::ModeChanged];
    actions.extend(match from {
        Mode::Command(_) => {
            unfocus_buffer(&mut commandline.cursor);
            update_commandline_on_mode_change(commandline, modes)
        }
        Mode::Insert | Mode::Navigation | Mode::Normal => {
            unfocus_buffer(&mut buffer.current_cursor);
            vec![]
        }
    });

    set_commandline_content_to_mode(commandline, register, modes);

    let msg = BufferMessage::ChangeMode(from.clone(), to.clone());
    actions.extend(match to {
        Mode::Command(_) => {
            focus_buffer(&mut commandline.cursor);
            update_commandline_on_mode_change(commandline, modes)
        }
        Mode::Insert => {
            focus_buffer(&mut buffer.current_cursor);
            update_current(layout, &modes.current, buffer, &msg);
            vec![]
        }
        Mode::Navigation => {
            // TODO: handle file operations: show pending with gray, refresh on operation success
            // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
            focus_buffer(&mut buffer.current_cursor);
            update_current(layout, &modes.current, buffer, &msg);
            persist_path_changes(junk, &modes.current, buffer)
        }
        Mode::Normal => {
            focus_buffer(&mut buffer.current_cursor);
            update_current(layout, &modes.current, buffer, &msg);
            vec![]
        }
    });

    actions
}

fn update_commandline_on_mode_change(
    commandline: &mut CommandLine,
    modes: &mut ModeState,
) -> Vec<Action> {
    let buffer = &mut commandline.buffer;
    let viewport = &mut commandline.viewport;

    set_viewport_dimensions(viewport, &commandline.layout.buffer);

    let command_mode = match &modes.current {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => {
            let from_command = modes
                .previous
                .as_ref()
                .is_some_and(|mode| mode.is_command());

            if from_command {
                update_buffer(
                    viewport,
                    &mut commandline.cursor,
                    &modes.current,
                    buffer,
                    &BufferMessage::SetContent(vec![]),
                );
            }
            return Vec::new();
        }
    };

    match command_mode {
        CommandMode::Command | CommandMode::Search(_) => {
            update_buffer(
                viewport,
                &mut commandline.cursor,
                &modes.current,
                buffer,
                &BufferMessage::ResetCursor,
            );

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
                &mut commandline.cursor,
                &modes.current,
                buffer,
                &BufferMessage::SetContent(vec![bufferline]),
            );
        }
        CommandMode::PrintMultiline => {}
    };

    Vec::new()
}

fn set_commandline_content_to_mode(
    commandline: &mut CommandLine,
    register: &Register,
    modes: &mut ModeState,
) {
    if let Some(RegisterScope::Macro(identifier)) = &get_macro_register(register) {
        set_recording_in_commandline(commandline, modes, *identifier);
    } else {
        set_mode_in_commandline(commandline, modes);
    };
}

pub fn set_recording_in_commandline(
    commandline: &mut CommandLine,
    modes: &mut ModeState,
    identifier: char,
) -> Vec<Action> {
    let content = format!("recording @{}", identifier);
    print_in_commandline(commandline, modes, &[PrintContent::Default(content)]);
    Vec::new()
}

pub fn set_mode_in_commandline(
    commandline: &mut CommandLine,
    modes: &mut ModeState,
) -> Vec<Action> {
    let content = format!("--{}--", modes.current.to_string().to_uppercase());
    print_in_commandline(commandline, modes, &[PrintContent::Default(content)]);
    Vec::new()
}
