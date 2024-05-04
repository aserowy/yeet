use std::{cmp::Ordering, collections::VecDeque};

use ratatui::layout::Rect;
use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::{viewport::ViewPort, BufferLine, Mode},
    update::update_buffer,
};
use yeet_keymap::message::{Envelope, KeySequence, Message, PrintContent};

use crate::{
    action::Action,
    model::{DirectoryBufferState, Model},
};

mod buffer;
mod command;
pub mod commandline;
mod current;
mod cursor;
mod enumeration;
mod junkyard;
mod mark;
mod navigation;
mod parent;
mod path;
mod preview;
mod qfix;
mod register;
mod search;
mod settings;

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
    commandline::update_commandline(model, None);

    register::start_scope(&model.mode, &mut model.register, envelope);

    let actions = envelope
        .messages
        .iter()
        .flat_map(|message| update_with_message(model, message))
        .collect();

    register::finish_scope(&model.mode, &mut model.register, envelope);

    actions
}

#[tracing::instrument(skip(model))]
fn update_with_message(model: &mut Model, message: &Message) -> Vec<Action> {
    settings::update(model);

    match message {
        Message::Buffer(msg) => buffer::update(model, msg),
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

            update_buffer(
                &model.mode,
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
        Message::ReplayMacro(char) => {
            if let Some(content) = model.register.get(char) {
                model.register.r#macro = Some(content.to_string());
                vec![Action::EmitMessages(vec![Message::ExecuteKeySequence(
                    content.to_string(),
                )])]
            } else {
                Vec::new()
            }
        }
        Message::SetMark(char) => {
            mark::add(model, *char);
            Vec::new()
        }
        Message::StartMacro(identifier) => {
            // NOTE: macro scopes are handled with register scopes
            commandline::set_content_to_macro(model, *identifier);
            Vec::new()
        }
        Message::StopMacro => {
            // NOTE: macro scopes are handled with register scopes
            commandline::set_content_to_mode(model);
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

fn set_viewport_dimensions(vp: &mut ViewPort, rect: &Rect) {
    vp.height = usize::from(rect.height);
    vp.width = usize::from(rect.width);
}
