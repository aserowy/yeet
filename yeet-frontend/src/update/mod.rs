use yeet_keymap::message::{self, CommandMode, Message, Mode, PrintContent};

use crate::{
    action::Action,
    model::{buffer::Buffer, Model},
    settings::Settings,
};

use self::model::{commandline, current, preview};

mod buffer;
mod command;
mod enumeration;
mod history;
pub mod model;
mod navigation;
mod path;
mod register;
mod search;

pub fn update(settings: &Settings, model: &mut Model, message: &Message) -> Option<Vec<Action>> {
    match message {
        Message::Buffer(msg) => buffer(model, msg),
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
        Message::NavigateToPath(path) => navigation::path(model, path),
        Message::NavigateToSelected => navigation::selected(model),
        Message::OpenSelected => current::open(model, settings),
        Message::PasteFromJunkYard(register) => register::paste(model, register),
        Message::PathRemoved(path) => path::remove(model, path),
        Message::PathsAdded(paths) => path::add(model, paths),
        Message::PathsWriteFinished(paths) => register::add(model, paths),
        Message::PreviewLoaded(path, content) => preview::update(model, path, content),
        Message::Print(content) => commandline::print(model, content),
        Message::Rerender => None,
        Message::Resize(x, y) => Some(vec![Action::Resize(*x, *y)]),
        Message::SearchAndSelect(is_next) => search::search_and_select(model, *is_next),
        Message::SetMark(char) => {
            let selected = current::selection(model);
            if let Some(selected) = selected {
                model.marks.entries.insert(*char, selected);
            }
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
            commandline::print(model, &[PrintContent::Info(content)]);

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

                if let Some(preview_actions) = preview::path(model, true, true) {
                    actions.extend(preview_actions);
                    model.preview.buffer.lines.clear();
                    preview::viewport(model);
                }

                Some(actions)
            }
            Mode::Navigation => None,
        },
        message::Buffer::MoveCursor(_, _) | message::Buffer::MoveViewPort(_) => match model.mode {
            Mode::Command(_) => Some(commandline::update(model, Some(msg))),
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
        message::Buffer::SaveBuffer(_) => Some(current::save_changes(model)),
    }
}
