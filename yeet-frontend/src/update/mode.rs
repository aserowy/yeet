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
        register::{Register, RegisterScope},
        CommandLine, FileTreeBuffer, ModeState, State,
    },
    update::update_current,
};

use super::{commandline, register::get_macro_register, save::changes, viewport::set_dimensions};

pub fn change(
    state: &mut State,
    commandline: &mut CommandLine,
    layout: &AppLayout,
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

    state.modes.current = to.clone();
    state.modes.previous = Some(from.clone());

    let mut actions = vec![Action::ModeChanged];
    actions.extend(match from {
        Mode::Command(_) => {
            unfocus_buffer(&mut commandline.cursor);
            update_commandline_on_mode_change(commandline, &mut state.modes)
        }
        Mode::Insert | Mode::Navigation | Mode::Normal => {
            unfocus_buffer(&mut buffer.current_cursor);
            vec![]
        }
    });

    set_commandline_content_to_mode(commandline, &state.register, &mut state.modes);

    let msg = BufferMessage::ChangeMode(from.clone(), to.clone());
    actions.extend(match to {
        Mode::Command(_) => {
            focus_buffer(&mut commandline.cursor);
            update_commandline_on_mode_change(commandline, &mut state.modes)
        }
        Mode::Insert => {
            focus_buffer(&mut buffer.current_cursor);
            update_current(layout, &state.modes.current, buffer, &msg);
            vec![]
        }
        Mode::Navigation => {
            // TODO: handle file operations: show pending with gray, refresh on operation success
            // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
            focus_buffer(&mut buffer.current_cursor);
            update_current(layout, &state.modes.current, buffer, &msg);
            changes(&mut state.junk, &state.modes.current, buffer)
        }
        Mode::Normal => {
            focus_buffer(&mut buffer.current_cursor);
            update_current(layout, &state.modes.current, buffer, &msg);
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

    set_dimensions(viewport, &commandline.layout.buffer);

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
        print_recording(commandline, modes, *identifier);
    } else {
        print_mode(commandline, modes);
    };
}

pub fn print_recording(
    commandline: &mut CommandLine,
    modes: &mut ModeState,
    identifier: char,
) -> Vec<Action> {
    let content = format!("recording @{}", identifier);
    commandline::print(commandline, modes, &[PrintContent::Default(content)]);
    Vec::new()
}

pub fn print_mode(commandline: &mut CommandLine, modes: &mut ModeState) -> Vec<Action> {
    let content = format!("--{}--", modes.current.to_string().to_uppercase());
    commandline::print(commandline, modes, &[PrintContent::Default(content)]);
    Vec::new()
}
