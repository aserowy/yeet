use yeet_buffer::{
    message::BufferMessage,
    model::{BufferLine, CommandMode, Mode, SearchDirection},
};
use yeet_keymap::message::PrintContent;

use crate::{
    action::Action,
    model::{
        register::{Register, RegisterScope},
        App, Buffer, CommandLine, ModeState, PendingPathEvent, State,
    },
};

use super::{app, commandline, junkyard, path, register::get_macro_register, save};

pub fn change(app: &mut App, state: &mut State, from: &Mode, to: &Mode) -> Vec<Action> {
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
            app.commandline.viewport.hide_cursor = true;
            update_commandline_on_mode_change(&mut app.commandline, &mut state.modes)
        }
        Mode::Insert | Mode::Navigation | Mode::Normal => {
            let (vp, _buffer) = app::get_focused_current_mut(app);
            vp.hide_cursor = true;

            vec![]
        }
    });

    set_commandline_content_to_mode(&mut app.commandline, &state.register, &mut state.modes);

    let msg = BufferMessage::ChangeMode(from.clone(), to.clone());
    actions.extend(match to {
        Mode::Command(_) => {
            app.commandline.viewport.hide_cursor = false;
            update_commandline_on_mode_change(&mut app.commandline, &mut state.modes)
        }
        Mode::Insert => {
            let (vp, buffer) = match app::get_focused_current_mut(app) {
                (vp, Buffer::Directory(it)) => (vp, it),
                (_vp, Buffer::Image(_)) => return Vec::new(),
                (_vp, Buffer::Content(_)) => return Vec::new(),
                (_vp, Buffer::Empty) => return Vec::new(),
            };

            vp.hide_cursor = false;

            yeet_buffer::update(
                Some(vp),
                &state.modes.current,
                &mut buffer.buffer,
                std::slice::from_ref(&msg),
            );

            vec![]
        }
        Mode::Navigation => {
            let (vp, buffer) = match app::get_focused_current_mut(app) {
                (vp, Buffer::Directory(it)) => (vp, it),
                (_vp, Buffer::Image(_)) => return Vec::new(),
                (_vp, Buffer::Content(_)) => return Vec::new(),
                (_vp, Buffer::Empty) => return Vec::new(),
            };

            // TODO: handle file operations: show pending with gray, refresh on operation success
            // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
            vp.hide_cursor = false;

            yeet_buffer::update(
                Some(vp),
                &state.modes.current,
                &mut buffer.buffer,
                std::slice::from_ref(&msg),
            );

            save::changes(app, &mut state.junk, &state.modes.current)
        }
        Mode::Normal => {
            let (vp, buffer) = match app::get_focused_current_mut(app) {
                (vp, Buffer::Directory(it)) => (vp, it),
                (_vp, Buffer::Image(_)) => return Vec::new(),
                (_vp, Buffer::Content(_)) => return Vec::new(),
                (_vp, Buffer::Empty) => return Vec::new(),
            };

            vp.hide_cursor = false;

            yeet_buffer::update(
                Some(vp),
                &state.modes.current,
                &mut buffer.buffer,
                std::slice::from_ref(&msg),
            );

            vec![]
        }
    });

    if matches!(from, Mode::Insert) {
        actions.extend(flush_pending_paths(state, app));
    }

    actions
}

fn flush_pending_paths(state: &mut State, app: &mut App) -> Vec<Action> {
    let mut actions = Vec::new();
    for event in state.pending_path_events.drain(..) {
        match event {
            PendingPathEvent::Added(paths) => {
                path::add(
                    &state.history,
                    &state.marks,
                    &state.qfix,
                    &state.modes.current,
                    app,
                    &paths,
                );
                actions.extend(junkyard::cleanup_if_path_in_junkyard(&mut state.junk, &paths));
            }
            PendingPathEvent::Removed(path) => {
                path::remove(
                    &mut state.history,
                    &mut state.marks,
                    &mut state.qfix,
                    &mut state.junk,
                    &state.modes.current,
                    app,
                    &path,
                );
            }
        }
    }
    actions
}

fn update_commandline_on_mode_change(
    commandline: &mut CommandLine,
    modes: &mut ModeState,
) -> Vec<Action> {
    let buffer = &mut commandline.buffer;
    let viewport = &mut commandline.viewport;

    let command_mode = match &modes.current {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => {
            let from_command = matches!(modes.previous.as_ref(), Some(mode) if mode.is_command());

            if from_command {
                let message = BufferMessage::SetContent(vec![]);
                yeet_buffer::update(
                    Some(viewport),
                    &modes.current,
                    buffer,
                    std::slice::from_ref(&message),
                );
            }
            return Vec::new();
        }
    };

    match command_mode {
        CommandMode::Command | CommandMode::Search(_) => {
            let message = BufferMessage::ResetCursor;
            yeet_buffer::update(
                Some(viewport),
                &modes.current,
                buffer,
                std::slice::from_ref(&message),
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

            let message = BufferMessage::SetContent(vec![bufferline]);
            yeet_buffer::update(
                Some(viewport),
                &modes.current,
                buffer,
                std::slice::from_ref(&message),
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

#[cfg(test)]
mod test {
    use yeet_buffer::model::Mode;

    use crate::{model::PendingPathEvent, update::mode};

    #[test]
    fn leaving_insert_flushes_pending_paths_in_order() {
        let mut app = crate::model::App::default();
        let mut state = crate::model::State::default();

        let added_paths = vec![
            std::path::PathBuf::from("/tmp/a"),
            std::path::PathBuf::from("/tmp/b"),
        ];
        let removed_path = std::path::PathBuf::from("/tmp/c");

        state
            .pending_path_events
            .push(PendingPathEvent::Added(added_paths));
        state
            .pending_path_events
            .push(PendingPathEvent::Removed(removed_path));

        state.modes.current = Mode::Insert;
        let actions = mode::change(&mut app, &mut state, &Mode::Insert, &Mode::Normal);

        assert!(actions
            .iter()
            .any(|action| matches!(action, crate::action::Action::ModeChanged)));
        assert!(state.pending_path_events.is_empty());
    }
}
