use yeet_keymap::message::{self, CommandMode, CursorDirection, Message, Mode, PrintContent};

use crate::{
    action::Action,
    model::{
        buffer::{Buffer, SignIdentifier},
        Model,
    },
};

use self::model::{commandline, current, preview};

mod buffer;
mod bufferline;
mod command;
mod enumeration;
mod history;
mod mark;
pub mod model;
mod navigation;
mod path;
mod qfix;
mod register;
mod search;

pub fn update(model: &mut Model, message: &Message) -> Option<Vec<Action>> {
    settings(model);

    match message {
        Message::Buffer(msg) => buffer(model, msg),
        Message::DeleteMarks(marks) => mark::delete(model, marks),
        Message::ClearSearchHighlight => {
            search::clear(model);
            None
        }
        Message::EnumerationChanged(path, contents) => enumeration::changed(model, path, contents),
        Message::EnumerationFinished(path) => enumeration::finished(model, path),
        Message::Error(error) => {
            // TODO: buffer messages till command mode left
            if !model.mode.is_command() {
                commandline::print(model, &[PrintContent::Error(error.to_string())]);
            }
            None
        }
        Message::ExecuteCommand => match &model.mode {
            Mode::Command(_) => commandline::update_on_execute(model),
            _ => None,
        },
        Message::ExecuteCommandString(command) => Some(command::execute(command, model)),
        Message::KeySequenceChanged(sequence) => {
            model.key_sequence = sequence.clone();
            commandline::update(model, None);

            Some(vec![
                Action::SkipRender,
                Action::EmitMessages(vec![Message::Rerender]),
            ])
        }
        Message::NavigateToMark(char) => {
            let path = match model.marks.entries.get(char) {
                Some(it) => it.clone(),
                None => return None,
            };

            navigation::path(model, &path)
        }
        Message::NavigateToParent => navigation::parent(model),
        Message::NavigateToPath(path) => {
            if path.is_dir() {
                navigation::path(model, path)
            } else {
                navigation::path_as_preview(model, path)
            }
        }
        Message::NavigateToPathAsPreview(path) => navigation::path_as_preview(model, path),
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

            Some(actions)
        }
        Message::PreviewLoaded(path, content) => preview::update(model, path, content),
        Message::Print(content) => commandline::print(model, content),
        Message::Rerender => None,
        Message::Resize(x, y) => Some(vec![Action::Resize(*x, *y)]),
        Message::SetMark(char) => {
            mark::add(model, *char);
            None
        }
        Message::ToggleQuickFix => {
            qfix::toggle(model);
            None
        }
        Message::Quit => Some(vec![Action::Quit(None)]),
        Message::YankToJunkYard(repeat) => register::yank(model, repeat),
    }
}

fn sort_content(mode: &Mode, model: &mut Buffer) {
    model.lines.sort_unstable_by(|a, b| {
        a.content
            .to_ascii_uppercase()
            .cmp(&b.content.to_ascii_uppercase())
    });
    buffer::cursor::validate(mode, model);
}

fn settings(model: &mut Model) {
    model.current.buffer.set(&model.settings.current);
    model.parent.buffer.set(&model.settings.parent);
    model.preview.buffer.set(&model.settings.preview);

    if model.settings.show_mark_signs {
        remove_hidden_sign_on_all_buffer(model, &SignIdentifier::Mark);
    } else {
        add_hidden_sign_on_all_buffer(model, SignIdentifier::Mark);
    }

    if model.settings.show_quickfix_signs {
        remove_hidden_sign_on_all_buffer(model, &SignIdentifier::QuickFix);
    } else {
        add_hidden_sign_on_all_buffer(model, SignIdentifier::QuickFix);
    }
}

fn add_hidden_sign_on_all_buffer(model: &mut Model, id: SignIdentifier) {
    add_hidden_sign(&mut model.current.buffer, id.clone());
    add_hidden_sign(&mut model.parent.buffer, id.clone());
    add_hidden_sign(&mut model.preview.buffer, id);
}

fn add_hidden_sign(buffer: &mut Buffer, id: SignIdentifier) {
    buffer.view_port.hidden_sign_ids.insert(id);
}

fn remove_hidden_sign_on_all_buffer(model: &mut Model, id: &SignIdentifier) {
    remove_hidden_sign(&mut model.current.buffer, id);
    remove_hidden_sign(&mut model.parent.buffer, id);
    remove_hidden_sign(&mut model.preview.buffer, id);
}

fn remove_hidden_sign(buffer: &mut Buffer, id: &SignIdentifier) {
    buffer.view_port.hidden_sign_ids.remove(id);
}

fn buffer(model: &mut Model, msg: &message::Buffer) -> Option<Vec<Action>> {
    match msg {
        message::Buffer::ChangeMode(from, to) => {
            match (from, to) {
                (Mode::Command(_), Mode::Command(_))
                | (Mode::Insert, Mode::Insert)
                | (Mode::Navigation, Mode::Navigation)
                | (Mode::Normal, Mode::Normal) => return None,
                _ => {}
            }

            model.mode = to.clone();
            model.mode_before = Some(from.clone());

            let mut actions = vec![Action::ModeChanged];
            actions.extend(match from {
                Mode::Command(_) => {
                    buffer::unfocus_buffer(&mut model.commandline.buffer);
                    commandline::update(model, Some(msg))
                }
                Mode::Insert | Mode::Navigation | Mode::Normal => {
                    buffer::unfocus_buffer(&mut model.current.buffer);
                    vec![]
                }
            });

            let content = format!("--{}--", to.to_string().to_uppercase());
            commandline::print(model, &[PrintContent::Default(content)]);

            actions.extend(match to {
                Mode::Command(_) => {
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
                    current::save_changes(model)
                }
                Mode::Normal => {
                    buffer::focus_buffer(&mut model.current.buffer);
                    current::update(model, Some(msg));
                    vec![]
                }
            });

            Some(actions)
        }
        message::Buffer::Modification(_, _) => match model.mode {
            Mode::Command(CommandMode::Command) => Some(commandline::update(model, Some(msg))),
            Mode::Command(_) => {
                let actions = commandline::update(model, Some(msg));
                search::update(model);

                Some(actions)
            }
            Mode::Insert | Mode::Normal => {
                let mut actions = Vec::new();
                current::update(model, Some(msg));

                if let Some(preview_actions) = preview::selected_path(model, true, true) {
                    actions.extend(preview_actions);
                    preview::viewport(model);
                }

                Some(actions)
            }
            Mode::Navigation => None,
        },
        message::Buffer::MoveCursor(_, mtn) => match model.mode {
            Mode::Command(_) => Some(commandline::update(model, Some(msg))),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                if matches!(mtn, &CursorDirection::Search(_)) {
                    search::search(model);
                }

                let mut actions = Vec::new();
                current::update(model, Some(msg));

                if let Some(preview_actions) = preview::selected_path(model, true, true) {
                    actions.extend(preview_actions);
                    preview::viewport(model);
                }

                Some(actions)
            }
        },
        message::Buffer::MoveViewPort(_) => match model.mode {
            Mode::Command(_) => Some(commandline::update(model, Some(msg))),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                let mut actions = Vec::new();
                current::update(model, Some(msg));

                if let Some(preview_actions) = preview::selected_path(model, true, true) {
                    actions.extend(preview_actions);
                    preview::viewport(model);
                }

                Some(actions)
            }
        },
        message::Buffer::SaveBuffer(_) => Some(current::save_changes(model)),
    }
}
