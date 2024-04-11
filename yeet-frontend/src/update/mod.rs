use std::{cmp::Ordering, collections::VecDeque};

use yeet_buffer::{
    message::{BufferMessage, CursorDirection, ViewPortDirection},
    model::{Buffer, BufferLine, CommandMode, Mode, SignIdentifier},
    update,
};
use yeet_keymap::message::{Envelope, KeySequence, Message, PrintContent};

use crate::{
    action::Action,
    model::{mark::MARK_SIGN_ID, qfix::QFIX_SIGN_ID, DirectoryBufferState, Model},
};

use self::model::{commandline, current, preview};

mod bufferline;
mod command;
mod cursor;
mod enumeration;
mod junkyard;
mod mark;
pub mod model;
mod navigation;
mod path;
mod qfix;
mod register;
mod search;

const SORT: fn(&BufferLine, &BufferLine) -> Ordering = |a, b| {
    a.content
        .to_ascii_uppercase()
        .cmp(&b.content.to_ascii_uppercase())
};

#[tracing::instrument(skip(model))]
pub fn update(model: &mut Model, envelope: &Envelope) -> Vec<Action> {
    match &envelope.sequence {
        KeySequence::Completed(_) => model.key_sequence.clear(),
        KeySequence::Changed(sequence) => model.key_sequence = sequence.to_owned(),
        KeySequence::None => {}
    };
    commandline::update(model, None);

    register::scope(&model.mode, &mut model.register, envelope);

    let actions = envelope
        .messages
        .iter()
        .flat_map(|message| update_with_message(model, message))
        .collect();

    register::finish(&model.mode, &mut model.register, envelope);

    actions
}

#[tracing::instrument(skip(model))]
fn update_with_message(model: &mut Model, message: &Message) -> Vec<Action> {
    update_settings(model);

    match message {
        Message::Buffer(msg) => buffer(model, msg),
        Message::ClearSearchHighlight => {
            search::clear(model);
            Vec::new()
        }
        Message::DeleteMarks(marks) => mark::delete(model, marks),
        Message::EnumerationChanged(path, contents, selection) => {
            enumeration::changed(model, path, contents, selection);

            let mut actions = Vec::new();
            if model.files.current.state != DirectoryBufferState::Loading {
                if let Some(path) = preview::selected_path(model) {
                    model.files.preview.state = DirectoryBufferState::Loading;
                    preview::viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }
            }

            actions
        }
        Message::EnumerationFinished(path, selection) => {
            enumeration::finished(model, path, selection);

            update::update(
                &model.mode,
                &model.search,
                &mut model.files.parent.buffer,
                &BufferMessage::MoveViewPort(ViewPortDirection::CenterOnCursor),
            );

            Vec::new()
        }
        Message::Error(error) => {
            // TODO: buffer messages till command mode left
            if !model.mode.is_command() {
                commandline::print(model, &[PrintContent::Error(error.to_string())]);
            }
            Vec::new()
        }
        Message::ExecuteCommand => match &model.mode {
            Mode::Command(_) => commandline::update_on_execute(model),
            _ => Vec::new(),
        },
        Message::ExecuteCommandString(command) => command::execute(command, model),
        Message::ExecuteKeySequence(_) => {
            if let Some(commands) = &mut model.command_stack {
                commands.push_back(message.clone());
            } else {
                let mut stack = VecDeque::new();
                stack.push_back(message.clone());
                model.command_stack = Some(stack);
            }
            Vec::new()
        }
        Message::ExecuteRegister(register) => {
            let key_sequence = model.register.get(register);
            match key_sequence {
                Some(key_sequence) => {
                    vec![Action::EmitMessages(vec![Message::ExecuteKeySequence(
                        key_sequence,
                    )])]
                }
                None => Vec::new(),
            }
        }
        Message::LeaveCommandMode => match &model.mode {
            Mode::Command(_) => commandline::update_on_leave(model),
            _ => Vec::new(),
        },
        Message::NavigateToMark(char) => {
            let path = match model.marks.entries.get(char) {
                Some(it) => it.clone(),
                None => return Vec::new(),
            };

            let selection = path
                .file_name()
                .map(|oss| oss.to_string_lossy().to_string());

            let path = match path.parent() {
                Some(parent) => parent,
                None => &path,
            };

            navigation::path(model, path, &selection)
        }
        Message::NavigateToParent => navigation::parent(model),
        Message::NavigateToPath(path) => {
            if path.is_dir() {
                navigation::path(model, path, &None)
            } else {
                let selection = path
                    .file_name()
                    .map(|oss| oss.to_string_lossy().to_string());

                let path = match path.parent() {
                    Some(parent) => parent,
                    None => path,
                };

                navigation::path(model, path, &selection)
            }
        }
        Message::NavigateToPathAsPreview(path) => {
            let selection = path
                .file_name()
                .map(|oss| oss.to_string_lossy().to_string());

            let path = match path.parent() {
                Some(parent) => parent,
                None => path,
            };

            navigation::path(model, path, &selection)
        }
        Message::NavigateToSelected => navigation::selected(model),
        Message::OpenSelected => current::open(model),
        Message::PasteFromJunkYard(entry_id) => junkyard::paste(model, entry_id),
        Message::PathRemoved(path) => {
            if path.starts_with(&model.junk.path) {
                model.junk.remove(path);
            }

            path::remove(model, path)
        }
        Message::PathsAdded(paths) => {
            let mut actions = path::add(model, paths);
            actions.extend(junkyard::add(model, paths));

            actions
        }
        Message::PreviewLoaded(path, content) => {
            preview::update(model, path, content);
            Vec::new()
        }
        Message::Print(content) => commandline::print(model, content),
        Message::Rerender => Vec::new(),
        Message::Resize(x, y) => vec![Action::Resize(*x, *y)],
        Message::SetMark(char) => {
            mark::add(model, *char);
            Vec::new()
        }
        Message::StartMacro(_) | Message::StopMacro => {
            // NOTE: macro scopes are handled with register scopes
            Vec::new()
        }
        Message::ToggleQuickFix => {
            qfix::toggle(model);
            Vec::new()
        }
        Message::Quit => vec![Action::Quit(None)],
        Message::YankToJunkYard(repeat) => junkyard::yank(model, repeat),
    }
}

fn update_settings(model: &mut Model) {
    model.files.current.buffer.set(&model.settings.current);
    model.files.parent.buffer.set(&model.settings.parent);
    model.files.preview.buffer.set(&model.settings.preview);

    if model.settings.show_mark_signs {
        remove_hidden_sign_on_all_buffer(model, &MARK_SIGN_ID);
    } else {
        add_hidden_sign_on_all_buffer(model, MARK_SIGN_ID);
    }

    if model.settings.show_quickfix_signs {
        remove_hidden_sign_on_all_buffer(model, &QFIX_SIGN_ID);
    } else {
        add_hidden_sign_on_all_buffer(model, QFIX_SIGN_ID);
    }
}

fn add_hidden_sign_on_all_buffer(model: &mut Model, id: SignIdentifier) {
    add_hidden_sign(&mut model.files.current.buffer, id);
    add_hidden_sign(&mut model.files.parent.buffer, id);
    add_hidden_sign(&mut model.files.preview.buffer, id);
}

fn add_hidden_sign(buffer: &mut Buffer, id: SignIdentifier) {
    buffer.view_port.hidden_sign_ids.insert(id);
}

fn remove_hidden_sign_on_all_buffer(model: &mut Model, id: &SignIdentifier) {
    remove_hidden_sign(&mut model.files.current.buffer, id);
    remove_hidden_sign(&mut model.files.parent.buffer, id);
    remove_hidden_sign(&mut model.files.preview.buffer, id);
}

fn remove_hidden_sign(buffer: &mut Buffer, id: &SignIdentifier) {
    buffer.view_port.hidden_sign_ids.remove(id);
}

#[tracing::instrument(skip(model, msg))]
fn buffer(model: &mut Model, msg: &BufferMessage) -> Vec<Action> {
    match msg {
        BufferMessage::ChangeMode(from, to) => {
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
                    update::unfocus(&mut model.commandline.buffer);
                    commandline::update_on_mode_change(model, from, to)
                }
                Mode::Insert | Mode::Navigation | Mode::Normal => {
                    update::unfocus(&mut model.files.current.buffer);
                    vec![]
                }
            });

            let content = format!("--{}--", to.to_string().to_uppercase());
            commandline::print(model, &[PrintContent::Default(content)]);

            actions.extend(match to {
                Mode::Command(_) => {
                    update::focus(&mut model.commandline.buffer);
                    commandline::update_on_mode_change(model, from, to)
                }
                Mode::Insert => {
                    update::focus(&mut model.files.current.buffer);
                    current::update(model, Some(msg));
                    vec![]
                }
                Mode::Navigation => {
                    // TODO: handle file operations: show pending with gray, refresh on operation success
                    // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
                    update::focus(&mut model.files.current.buffer);
                    current::update(model, Some(msg));
                    current::save_changes(model)
                }
                Mode::Normal => {
                    update::focus(&mut model.files.current.buffer);
                    current::update(model, Some(msg));
                    vec![]
                }
            });

            actions
        }
        BufferMessage::Modification(repeat, modification) => match model.mode {
            Mode::Command(CommandMode::Command) => {
                commandline::update_on_modification(model, repeat, modification)
            }
            Mode::Command(_) => {
                let actions = commandline::update_on_modification(model, repeat, modification);
                search::update(model);

                actions
            }
            Mode::Insert | Mode::Normal => {
                current::update(model, Some(msg));

                let mut actions = Vec::new();
                if let Some(path) = preview::selected_path(model) {
                    preview::viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }

                actions
            }
            Mode::Navigation => Vec::new(),
        },
        BufferMessage::MoveCursor(_, mtn) => match &model.mode {
            Mode::Command(_) => commandline::update(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                if matches!(mtn, &CursorDirection::Search(_)) {
                    let search = model.search.as_ref().map(|srch| srch.last.to_string());
                    model.register.searched = search;

                    search::search(model);
                }

                current::update(model, Some(msg));

                let mut actions = Vec::new();
                if let Some(path) = preview::selected_path(model) {
                    preview::viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }

                actions
            }
        },
        BufferMessage::MoveViewPort(_) => match model.mode {
            Mode::Command(_) => commandline::update(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                current::update(model, Some(msg));

                let mut actions = Vec::new();
                if let Some(path) = preview::selected_path(model) {
                    preview::viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }

                actions
            }
        },
        BufferMessage::SaveBuffer => current::save_changes(model),

        BufferMessage::RemoveLine(_)
        | BufferMessage::ResetCursor
        | BufferMessage::SetContent(_)
        | BufferMessage::SetCursorToLineContent(_)
        | BufferMessage::SortContent(_) => unreachable!(),
    }
}
