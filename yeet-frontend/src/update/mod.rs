use std::cmp::Ordering;

use yeet_buffer::{
    message::BufferMessage,
    model::{BufferLine, Mode},
};
use yeet_keymap::message::{KeySequence, KeymapMessage, PrintContent};
use yeet_lua::LuaConfiguration;

use crate::{
    action::Action,
    event::{Envelope, LogSeverity, Message},
    model::{App, Buffer, Model, State},
    settings::Settings,
    terminal::TerminalWrapper,
};

pub mod app;
mod buffers;
mod command;
pub mod commandline;
mod cursor;
mod enumeration;
mod focus;
pub mod history;
pub mod hook;
pub mod junkyard;
mod mark;
mod mode;
mod modify;
mod navigate;
mod open;
mod path;
mod preview;
mod qfix;
mod register;
mod save;
mod search;
mod selection;
mod settings;
mod sign;
mod tab;
mod task;
mod viewport;
pub mod window;

const SORT: fn(&BufferLine, &BufferLine) -> Ordering = |a, b| {
    a.content
        .to_stripped_string()
        .to_ascii_uppercase()
        .cmp(&b.content.to_stripped_string().to_ascii_uppercase())
};

#[tracing::instrument(skip(model, terminal))]
pub fn model(terminal: &TerminalWrapper, model: &mut Model, envelope: Envelope) -> Vec<Action> {
    match &envelope.sequence {
        KeySequence::Completed(_) => model.app.commandline.key_sequence.clear(),
        KeySequence::Changed(sequence) => {
            sequence.clone_into(&mut model.app.commandline.key_sequence)
        }
        KeySequence::None => {}
    };

    settings::update(model);
    commandline::update(&mut model.app.commandline, &model.state.modes.current, None);

    let keymaps: Vec<_> = envelope.clone_keymap_messages();
    register::start_scope(
        &model.state.modes.current,
        &mut model.state.register,
        &keymaps,
    );

    let sequence = envelope.sequence.clone();
    let actions = envelope
        .messages
        .into_iter()
        .flat_map(|message| {
            update_with_message(
                &mut model.app,
                &mut model.state,
                &model.settings,
                model.lua.as_ref(),
                message,
            )
        })
        .collect();

    let size = terminal.size().expect("Failed to get terminal size");
    match window::update(&mut model.app, size) {
        Ok(_) => {}
        Err(err) => tracing::error!("window update failed with error: {}", err),
    };

    commandline::force_cursor_after_size_update(
        &mut model.app.commandline,
        &model.state.modes.current,
    );
    buffers::update(&mut model.app);

    if let Some(lua) = model.lua.as_ref() {
        if let Ok(window) = model.app.current_window() {
            if let Some((_, current_id, preview_id)) = app::get_focused_directory_buffer_ids(window)
            {
                let current_path = model
                    .app
                    .contents
                    .buffers
                    .get(&current_id)
                    .and_then(|buffer| buffer.resolve_path())
                    .map(|p| p.to_path_buf());

                let is_directory = model
                    .app
                    .contents
                    .buffers
                    .get(&preview_id)
                    .map(|b| matches!(b, Buffer::Directory(d) if d.path.is_dir()))
                    .unwrap_or(false);

                if let Ok(window) = model.app.current_window_mut() {
                    if let Some((parent, current, preview)) =
                        app::get_focused_directory_viewports_mut(window)
                    {
                        yeet_lua::invoke_on_window_change(
                            lua,
                            current_path.as_deref(),
                            &mut [parent, current, preview],
                            is_directory,
                        );
                    }
                }
            }
        }
    }

    register::finish_scope(
        &model.state.modes.current,
        &mut model.state.register,
        &sequence,
        &keymaps,
    );

    actions
}

#[tracing::instrument(skip_all)]
fn update_with_message(
    app: &mut App,
    state: &mut State,
    settings: &Settings,
    lua: Option<&LuaConfiguration>,
    message: Message,
) -> Vec<Action> {
    match message {
        Message::EnumerationChanged(path, contents, selection) => {
            match enumeration::change(
                state,
                app,
                &path,
                &contents,
                &selection,
                &settings.theme,
                lua,
            ) {
                Ok(actions) => actions,
                Err(err) => {
                    tracing::error!("EnumerationChanged failed: {}", err);
                    Vec::new()
                }
            }
        }
        Message::EnumerationFinished(path, contents, selection) => {
            match enumeration::finish(
                state,
                app,
                &path,
                &contents,
                &selection,
                &settings.theme,
                lua,
            ) {
                Ok(actions) => actions,
                Err(err) => {
                    tracing::error!("EnumerationFinished failed: {}", err);
                    Vec::new()
                }
            }
        }
        Message::Log(severity, msg) => {
            let content = match severity {
                LogSeverity::Error => PrintContent::Error(msg.to_string()),
                LogSeverity::Warning => PrintContent::Warning(msg.to_string()),
                LogSeverity::Information => PrintContent::Information(msg.to_string()),
            };
            commandline::print(
                &mut app.commandline,
                &mut state.modes,
                &[content],
                &settings.theme,
            )
        }
        Message::FdResult(paths) | Message::RgResult(paths) => qfix::add(
            &mut state.qfix,
            app.contents.buffers.values_mut().collect(),
            paths,
            &settings.theme,
        ),
        Message::Keymap(msg) => update_with_keymap_message(app, state, settings, lua, &msg),
        Message::QuickFixChanged => {
            command::qfix::window::refresh_quickfix_buffer(
                &mut app.tabs,
                &mut app.contents,
                &state.qfix,
                lua,
            );
            Vec::new()
        }
        Message::PathRemoved(path) => {
            if state.modes.current == Mode::Insert {
                state
                    .pending_path_events
                    .push(crate::model::PendingPathEvent::Removed(path));
                Vec::new()
            } else {
                match path::remove(
                    &mut state.history,
                    &mut state.marks,
                    &mut state.qfix,
                    &mut state.junk,
                    &state.modes.current,
                    app,
                    &path,
                ) {
                    Ok(actions) => actions,
                    Err(err) => {
                        tracing::error!("PathsRemoved failed: {}", err);
                        Vec::new()
                    }
                }
            }
        }
        Message::PathsAdded(paths) => {
            if state.modes.current == Mode::Insert {
                state
                    .pending_path_events
                    .push(crate::model::PendingPathEvent::Added(paths));
                Vec::new()
            } else {
                let mut actions = match path::add(
                    &mut state.history,
                    &state.marks,
                    &state.qfix,
                    &state.modes.current,
                    app,
                    &paths,
                    &settings.theme,
                    lua,
                ) {
                    Ok(actions) => actions,
                    Err(err) => {
                        tracing::error!("PathsAdded failed: {}", err);
                        Vec::new()
                    }
                };
                actions.extend(junkyard::cleanup_if_path_in_junkyard(
                    &mut state.junk,
                    &paths,
                ));
                actions
            }
        }
        Message::HelpHighlighted(buffer_id, lines) => {
            command::help::apply_highlighted(app, lua, buffer_id, lines);
            Vec::new()
        }
        Message::PreviewLoaded(content) => preview::update(app, lua, content),
        Message::Rerender => Vec::new(),
        Message::Resize(x, y) => vec![Action::Resize(x, y)],
        Message::TaskStarted(id, cancellation) => match app.current_window_and_contents_mut() {
            Ok((window, contents)) => {
                task::add(&mut state.tasks, window, contents, id, cancellation, lua)
            }
            Err(_) => Vec::new(),
        },
        Message::TaskEnded(id) => match app.current_window_and_contents_mut() {
            Ok((window, contents)) => task::remove(&mut state.tasks, window, contents, id, lua),
            Err(_) => Vec::new(),
        },
        Message::ZoxideResult(path) => navigate::path(app, &mut state.history, path.as_ref()),
    }
}

#[tracing::instrument(skip_all)]
pub fn update_with_keymap_message(
    app: &mut App,
    state: &mut State,
    settings: &Settings,
    lua: Option<&LuaConfiguration>,
    msg: &KeymapMessage,
) -> Vec<Action> {
    match msg {
        KeymapMessage::Buffer(msg) => update_with_buffer_message(app, state, settings, lua, msg),
        KeymapMessage::ClearSearchHighlight => {
            search::clear(app.contents.buffers.values_mut().collect());
            Vec::new()
        }
        KeymapMessage::DeleteMarks(mrks) => mark::delete(
            &mut state.marks,
            app.contents.buffers.values_mut().collect(),
            mrks,
        ),
        KeymapMessage::FocusDirection(direction) => focus::change(app, direction),
        KeymapMessage::ExecuteCommand => {
            commandline::update_on_execute(app, &mut state.register, &mut state.modes)
        }
        KeymapMessage::ExecuteCommandString(command) => {
            command::execute(app, state, settings, lua, command)
        }
        KeymapMessage::ExecuteKeySequence(key_sequence) => {
            state.remaining_keysequence.replace(key_sequence.clone());
            Vec::new()
        }
        KeymapMessage::ExecuteRegister(rgstr) => register::replay(&mut state.register, rgstr),
        KeymapMessage::LeaveCommandMode => {
            commandline::leave(app, &mut state.register, &state.modes)
        }
        KeymapMessage::NavigateToMark(char) => {
            navigate::mark(app, &mut state.history, &state.marks, char)
        }
        KeymapMessage::NavigateToParent => match navigate::parent(app) {
            Ok(actions) => actions,
            Err(err) => {
                tracing::error!("NavigateToParent failed: {}", err);
                Vec::new()
            }
        },
        KeymapMessage::NavigateToPath(path) => navigate::path(app, &mut state.history, path),
        KeymapMessage::NavigateToPathAsPreview(path) => {
            navigate::path_as_preview(app, &mut state.history, path)
        }
        KeymapMessage::NavigateToSelected => match navigate::selected(app, &mut state.history) {
            Ok(actions) => actions,
            Err(err) => {
                tracing::error!("NavigateToSelected failed: {}", err);
                Vec::new()
            }
        },
        KeymapMessage::OpenSelected => {
            match open::selected(settings, &state.modes.current, app, lua, &mut state.qfix) {
                Ok(actions) => actions,
                Err(err) => {
                    tracing::error!("OpenSelected failed: {}", err);
                    Vec::new()
                }
            }
        }
        KeymapMessage::PasteFromJunkYard(entry_id) => {
            match junkyard::paste(app, &state.junk, entry_id) {
                Ok(actions) => actions,
                Err(err) => {
                    tracing::error!("PasteFromJunkYard failed: {}", err);
                    Vec::new()
                }
            }
        }
        KeymapMessage::Print(content) => commandline::print(
            &mut app.commandline,
            &mut state.modes,
            content,
            &settings.theme,
        ),
        KeymapMessage::ReplayMacro(char) => register::replay_macro(&mut state.register, char),
        KeymapMessage::SetMark(char) => {
            match mark::add(app, &mut state.marks, *char, &settings.theme) {
                Ok(actions) => actions,
                Err(err) => {
                    tracing::error!("SetMark failed: {}", err);
                    Vec::new()
                }
            }
        }
        KeymapMessage::StartMacro(identifier) => mode::print_recording(
            &mut app.commandline,
            &mut state.modes,
            *identifier,
            &settings.theme,
        ),
        KeymapMessage::StopMacro => {
            mode::print_mode(&mut app.commandline, &mut state.modes, &settings.theme)
        }
        KeymapMessage::ToggleQuickFix => qfix::toggle(app, &mut state.qfix, &settings.theme),
        KeymapMessage::Quit(mode) => vec![Action::Quit(mode.clone(), None)],
        KeymapMessage::YankPathToClipboard => {
            let (window, contents) = match app.current_window_and_contents_mut() {
                Ok(window) => window,
                Err(_) => return Vec::new(),
            };
            let (current_vp, current_buffer) = match app::get_focused_current_mut(window, contents)
            {
                Ok((current_vp, current_buffer)) => (current_vp, current_buffer),
                Err(_) => return Vec::new(),
            };
            let directory = match current_buffer {
                Buffer::Directory(directory) => directory,
                _ => return Vec::new(),
            };

            selection::copy_to_clipboard(&mut state.register, directory, &current_vp.cursor)
        }
        KeymapMessage::YankToJunkYard(repeat) => {
            match junkyard::yank(app, &mut state.junk, repeat) {
                Ok(actions) => actions,
                Err(err) => {
                    tracing::error!("YankToJunkYard failed: {}", err);
                    Vec::new()
                }
            }
        }
    }
}

#[tracing::instrument(skip_all)]
pub fn update_with_buffer_message(
    app: &mut App,
    state: &mut State,
    settings: &Settings,
    lua: Option<&LuaConfiguration>,
    msg: &BufferMessage,
) -> Vec<Action> {
    match msg {
        BufferMessage::ChangeMode(from, to) => {
            match mode::change(app, state, from, to, &settings.theme, lua) {
                Ok(actions) => actions,
                Err(err) => {
                    tracing::error!("ChangeMode failed: {}", err);
                    Vec::new()
                }
            }
        }
        BufferMessage::Modification(repeat, modification) => match &mut state.modes.current {
            Mode::Command(_) => commandline::modify(app, &mut state.modes, repeat, modification),
            Mode::Insert | Mode::Normal => {
                match modify::buffer(app, state, lua, repeat, modification) {
                    Ok(actions) => actions,
                    Err(err) => {
                        tracing::error!("Modification failed: {}", err);
                        Vec::new()
                    }
                }
            }
            Mode::Navigation => {
                let vp = match app.current_window() {
                    Ok(window) => window.focused_viewport(),
                    Err(_) => return Vec::new(),
                };
                if matches!(
                    app.contents.buffers.get(&vp.buffer_id),
                    Some(Buffer::Tasks(_)) | Some(Buffer::QuickFix(_))
                ) {
                    match modify::buffer(app, state, lua, repeat, modification) {
                        Ok(actions) => actions,
                        Err(err) => {
                            tracing::error!("Modification failed: {}", err);
                            Vec::new()
                        }
                    }
                } else {
                    Vec::new()
                }
            }
        },
        BufferMessage::MoveCursor(rpt, mtn) => match &mut state.modes.current {
            Mode::Command(_) => {
                commandline::update(&mut app.commandline, &state.modes.current, Some(msg))
            }
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                match cursor::relocate(app, state, rpt, mtn) {
                    Ok(actions) => actions,
                    Err(err) => {
                        tracing::error!("MoveCursor failed: {}", err);
                        Vec::new()
                    }
                }
            }
        },
        BufferMessage::MoveViewPort(mtn) => match &state.modes.current {
            Mode::Command(_) => {
                commandline::update(&mut app.commandline, &state.modes.current, Some(msg))
            }
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                match viewport::relocate(app, &mut state.history, &state.modes.current, mtn) {
                    Ok(actions) => actions,
                    Err(err) => {
                        tracing::error!("MoveViewPort failed: {}", err);
                        Vec::new()
                    }
                }
            }
        },
        BufferMessage::SaveBuffer => {
            match save::current(app, &mut state.junk, &state.modes.current) {
                Ok(actions) => actions,
                Err(err) => {
                    tracing::error!("SaveBuffer failed: {}", err);
                    Vec::new()
                }
            }
        }

        BufferMessage::AddLine(_, _)
        | BufferMessage::RemoveLine(_)
        | BufferMessage::ResetCursor
        | BufferMessage::SetContent(_)
        | BufferMessage::SetCursorToLineContent(_)
        | BufferMessage::SortContent(_)
        | BufferMessage::UpdateViewPortByCursor => unreachable!(),
    }
}
