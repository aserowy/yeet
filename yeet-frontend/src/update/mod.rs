use std::cmp::Ordering;

use yeet_buffer::{
    message::{BufferMessage, CursorDirection, ViewPortDirection},
    model::{Buffer, BufferLine, CommandMode, Mode, SignIdentifier},
    update,
};
use yeet_keymap::message::{Message, PrintContent};

use crate::{
    action::Action,
    model::{mark::MARK_SIGN_ID, qfix::QFIX_SIGN_ID, DirectoryBufferState, Model},
};

use self::model::{commandline, current, preview};

mod bufferline;
mod command;
mod cursor;
mod enumeration;
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
pub fn update(model: &mut Model, message: &Message) -> Vec<Action> {
    settings(model);

    match message {
        Message::Buffer(msg) => buffer(model, msg),
        Message::DeleteMarks(marks) => mark::delete(model, marks),
        Message::ClearSearchHighlight => {
            search::clear(model);
            Vec::new()
        }
        Message::EnumerationChanged(path, contents, selection) => {
            enumeration::changed(model, path, contents, selection);

            let mut actions = Vec::new();
            if model.file_buffer.current.state != DirectoryBufferState::Loading {
                if let Some(path) = preview::selected_path(model) {
                    model.file_buffer.preview.state = DirectoryBufferState::Loading;
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
                &mut model.file_buffer.parent.buffer,
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
        Message::KeySequenceChanged(sequence) => {
            model.key_sequence = sequence.clone();
            commandline::update(model, None);

            vec![
                Action::SkipRender,
                Action::EmitMessages(vec![Message::Rerender]),
            ]
        }
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
        Message::PasteFromJunkYard(register) => register::paste(model, register),
        Message::PathRemoved(path) => {
            if path.starts_with(&model.junk.path) {
                model.junk.remove(path);
            }

            path::remove(model, path)
        }
        Message::PathsAdded(paths) => {
            let mut actions = path::add(model, paths);
            actions.extend(register::add(model, paths));

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
        Message::ToggleQuickFix => {
            qfix::toggle(model);
            Vec::new()
        }
        Message::Quit => vec![Action::Quit(None)],
        Message::YankToJunkYard(repeat) => register::yank(model, repeat),
    }
}

fn settings(model: &mut Model) {
    model
        .file_buffer
        .current
        .buffer
        .set(&model.settings.current);
    model.file_buffer.parent.buffer.set(&model.settings.parent);
    model
        .file_buffer
        .preview
        .buffer
        .set(&model.settings.preview);

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
    add_hidden_sign(&mut model.file_buffer.current.buffer, id);
    add_hidden_sign(&mut model.file_buffer.parent.buffer, id);
    add_hidden_sign(&mut model.file_buffer.preview.buffer, id);
}

fn add_hidden_sign(buffer: &mut Buffer, id: SignIdentifier) {
    buffer.view_port.hidden_sign_ids.insert(id);
}

fn remove_hidden_sign_on_all_buffer(model: &mut Model, id: &SignIdentifier) {
    remove_hidden_sign(&mut model.file_buffer.current.buffer, id);
    remove_hidden_sign(&mut model.file_buffer.parent.buffer, id);
    remove_hidden_sign(&mut model.file_buffer.preview.buffer, id);
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
                    commandline::update(model, Some(msg))
                }
                Mode::Insert | Mode::Navigation | Mode::Normal => {
                    update::unfocus(&mut model.file_buffer.current.buffer);
                    vec![]
                }
            });

            let content = format!("--{}--", to.to_string().to_uppercase());
            commandline::print(model, &[PrintContent::Default(content)]);

            actions.extend(match to {
                Mode::Command(_) => {
                    update::focus(&mut model.commandline.buffer);
                    commandline::update(model, Some(msg))
                }
                Mode::Insert => {
                    update::focus(&mut model.file_buffer.current.buffer);
                    current::update(model, Some(msg));
                    vec![]
                }
                Mode::Navigation => {
                    // TODO: handle file operations: show pending with gray, refresh on operation success
                    // TODO: sort and refresh current on PathEnumerationFinished while not in Navigation mode
                    update::focus(&mut model.file_buffer.current.buffer);
                    current::update(model, Some(msg));
                    current::save_changes(model)
                }
                Mode::Normal => {
                    update::focus(&mut model.file_buffer.current.buffer);
                    current::update(model, Some(msg));
                    vec![]
                }
            });

            actions
        }
        BufferMessage::Modification(_, _) => match model.mode {
            Mode::Command(CommandMode::Command) => commandline::update(model, Some(msg)),
            Mode::Command(_) => {
                let actions = commandline::update(model, Some(msg));
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
        BufferMessage::MoveCursor(_, mtn) => match model.mode {
            Mode::Command(_) => commandline::update(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                if matches!(mtn, &CursorDirection::Search(_)) {
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
