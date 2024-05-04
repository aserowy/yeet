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

use self::{
    buffer::update_with_buffer_message,
    command::execute_command,
    commandline::{
        leave_commandline, print_in_commandline, set_mode_in_commandline,
        set_recording_in_commandline, update_commandline, update_commandline_on_execute,
    },
    current::open_selected,
    enumeration::{update_on_enumeration_change, update_on_enumeration_finished},
    junkyard::{add_to_junkyard, paste_to_junkyard, yank_to_junkyard},
    mark::{add_mark, delete_mark},
    navigation::{navigate_to_parent, navigate_to_path, navigate_to_selected},
    path::{add_paths, remove_path},
    preview::{set_preview_to_selected, update_preview, validate_preview_viewport},
    qfix::toggle_selected_to_qfix,
    register::{finish_register_scope, start_register_scope},
    search::clear_search,
    settings::update_with_settings,
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
    update_commandline(model, None);
    update_with_settings(model);

    start_register_scope(&model.mode, &mut model.register, envelope);

    let actions = envelope
        .messages
        .iter()
        .flat_map(|message| update_with_message(model, message))
        .collect();

    finish_register_scope(&model.mode, &mut model.register, envelope);

    actions
}

#[tracing::instrument(skip(model))]
fn update_with_message(model: &mut Model, message: &Message) -> Vec<Action> {
    match message {
        Message::Buffer(msg) => update_with_buffer_message(model, msg),
        Message::ClearSearchHighlight => {
            clear_search(model);
            Vec::new()
        }
        Message::DeleteMarks(marks) => delete_mark(model, marks),
        Message::EnumerationChanged(path, contents, selection) => {
            update_on_enumeration_change(model, path, contents, selection);

            let mut actions = Vec::new();
            if model.files.current.state != DirectoryBufferState::Loading {
                if let Some(path) = set_preview_to_selected(model) {
                    model.files.preview.state = DirectoryBufferState::Loading;
                    validate_preview_viewport(model);

                    let selection = model.history.get_selection(&path).map(|s| s.to_owned());
                    actions.push(Action::Load(path, selection));
                }
            }

            actions
        }
        Message::EnumerationFinished(path, selection) => {
            update_on_enumeration_finished(model, path, selection);

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
                print_in_commandline(model, &[PrintContent::Error(error.to_string())]);
            }
            Vec::new()
        }
        Message::ExecuteCommand => match &model.mode {
            Mode::Command(_) => update_commandline_on_execute(model),
            _ => Vec::new(),
        },
        Message::ExecuteCommandString(command) => execute_command(command, model),
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
            Mode::Command(_) => leave_commandline(model),
            _ => Vec::new(),
        },
        Message::NavigateToMark(char) => {
            // TODO: move logic into to mark fn
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

            navigate_to_path(model, path, &selection)
        }
        Message::NavigateToParent => navigate_to_parent(model),
        Message::NavigateToPath(path) => {
            // TODO: move logic into to path fn
            if path.is_dir() {
                navigate_to_path(model, path, &None)
            } else {
                let selection = path
                    .file_name()
                    .map(|oss| oss.to_string_lossy().to_string());

                let path = match path.parent() {
                    Some(parent) => parent,
                    None => path,
                };

                navigate_to_path(model, path, &selection)
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

            navigate_to_path(model, path, &selection)
        }
        Message::NavigateToSelected => navigate_to_selected(model),
        Message::OpenSelected => open_selected(model),
        Message::PasteFromJunkYard(entry_id) => paste_to_junkyard(model, entry_id),
        Message::PathRemoved(path) => {
            if path.starts_with(&model.junk.path) {
                model.junk.remove(path);
            }

            remove_path(model, path)
        }
        Message::PathsAdded(paths) => {
            let mut actions = add_paths(model, paths);
            actions.extend(add_to_junkyard(model, paths));

            actions
        }
        Message::PreviewLoaded(path, content) => {
            update_preview(model, path, content);
            Vec::new()
        }
        Message::Print(content) => print_in_commandline(model, content),
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
            add_mark(model, *char);
            Vec::new()
        }
        Message::StartMacro(identifier) => {
            // NOTE: macro scopes are handled with register scopes
            set_recording_in_commandline(model, *identifier);
            Vec::new()
        }
        Message::StopMacro => {
            // NOTE: macro scopes are handled with register scopes
            set_mode_in_commandline(model);
            Vec::new()
        }
        Message::ToggleQuickFix => {
            toggle_selected_to_qfix(model);
            Vec::new()
        }
        Message::Quit => vec![Action::Quit(None)],
        Message::YankToJunkYard(repeat) => yank_to_junkyard(model, repeat),
    }
}

fn set_viewport_dimensions(vp: &mut ViewPort, rect: &Rect) {
    vp.height = usize::from(rect.height);
    vp.width = usize::from(rect.width);
}
