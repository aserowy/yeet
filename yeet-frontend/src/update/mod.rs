use std::{cmp::Ordering, collections::VecDeque};

use ratatui::layout::Rect;
use yeet_buffer::model::{viewport::ViewPort, BufferLine};
use yeet_keymap::message::{Envelope, KeySequence, Message, PrintContent};

use crate::{action::Action, model::Model};

use self::{
    buffer::update_with_buffer_message,
    command::execute_command,
    commandline::{
        leave_commandline, print_in_commandline, set_mode_in_commandline,
        set_recording_in_commandline, update_commandline, update_commandline_on_execute,
    },
    enumeration::{update_on_enumeration_change, update_on_enumeration_finished},
    junkyard::{add_to_junkyard, paste_to_junkyard, yank_to_junkyard},
    mark::{add_mark, delete_mark},
    navigation::{
        navigate_to_mark, navigate_to_parent, navigate_to_path, navigate_to_preview_path,
        navigate_to_selected,
    },
    open::open_selected,
    path::{add_paths, remove_path},
    preview::update_preview,
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
mod open;
mod parent;
mod path;
mod preview;
mod qfix;
mod register;
mod save;
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
        Message::ClearSearchHighlight => clear_search(model),
        Message::DeleteMarks(marks) => delete_mark(model, marks),
        Message::EnumerationChanged(path, contents, selection) => {
            update_on_enumeration_change(model, path, contents, selection)
        }
        Message::EnumerationFinished(path, selection) => {
            update_on_enumeration_finished(model, path, selection)
        }
        Message::Error(error) => {
            print_in_commandline(model, &[PrintContent::Error(error.to_string())])
        }
        Message::ExecuteCommand => update_commandline_on_execute(model),
        Message::ExecuteCommandString(command) => execute_command(command, model),
        // TODO: refactor into own function
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
        // TODO: refactor into own function
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
        Message::LeaveCommandMode => leave_commandline(model),
        Message::NavigateToMark(char) => navigate_to_mark(char, model),
        Message::NavigateToParent => navigate_to_parent(model),
        Message::NavigateToPath(path) => navigate_to_path(model, path),
        Message::NavigateToPathAsPreview(path) => navigate_to_preview_path(model, path),
        Message::NavigateToSelected => navigate_to_selected(model),
        Message::OpenSelected => open_selected(model),
        Message::PasteFromJunkYard(entry_id) => paste_to_junkyard(model, entry_id),
        Message::PathRemoved(path) => remove_path(model, path),
        Message::PathsAdded(paths) => add_paths(model, paths)
            .into_iter()
            .chain(add_to_junkyard(model, paths).into_iter())
            .collect(),
        Message::PreviewLoaded(path, content) => update_preview(model, path, content),
        Message::Print(content) => print_in_commandline(model, content),
        Message::Rerender => Vec::new(),
        Message::Resize(x, y) => vec![Action::Resize(*x, *y)],
        // TODO: refactor into own function
        Message::ReplayMacro(char) => {
            if let Some(content) = model.register.get(char) {
                model.register.last_macro = Some(content.to_string());
                vec![Action::EmitMessages(vec![Message::ExecuteKeySequence(
                    content.to_string(),
                )])]
            } else {
                Vec::new()
            }
        }
        Message::SetMark(char) => add_mark(model, *char),
        Message::StartMacro(identifier) => set_recording_in_commandline(model, *identifier),
        Message::StopMacro => set_mode_in_commandline(model),
        Message::ToggleQuickFix => toggle_selected_to_qfix(model),
        Message::Quit => vec![Action::Quit(None)],
        Message::YankToJunkYard(repeat) => yank_to_junkyard(model, repeat),
    }
}

fn set_viewport_dimensions(vp: &mut ViewPort, rect: &Rect) {
    vp.height = usize::from(rect.height);
    vp.width = usize::from(rect.width);
}
