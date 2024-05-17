use std::cmp::Ordering;

use yeet_buffer::{
    message::BufferMessage,
    model::{BufferLine, Mode},
    update::update_buffer,
};
use yeet_keymap::message::{Envelope, KeySequence, Message, PrintContent};

use crate::{action::Action, model::Model};

use self::{
    command::{create_or_extend_command_stack, execute_command},
    commandline::{
        leave_commandline, print_in_commandline, update_commandline, update_commandline_on_execute,
        update_commandline_on_modification,
    },
    cursor::move_cursor,
    enumeration::{update_on_enumeration_change, update_on_enumeration_finished},
    junkyard::{add_to_junkyard, paste_to_junkyard, yank_to_junkyard},
    mark::{add_mark, delete_mark},
    mode::{change_mode, set_mode_in_commandline, set_recording_in_commandline},
    modification::modify_buffer,
    navigation::{
        navigate_to_mark, navigate_to_parent, navigate_to_path, navigate_to_path_as_preview,
        navigate_to_selected,
    },
    open::open_selected,
    path::{add_paths, remove_path},
    preview::update_preview,
    qfix::toggle_selected_to_qfix,
    register::{
        finish_register_scope, replay_macro_register, replay_register, start_register_scope,
    },
    save::persist_path_changes,
    search::clear_search,
    selection::copy_current_selected_path_to_clipboard,
    settings::update_with_settings,
    viewport::{move_viewport, set_viewport_dimensions},
};

mod command;
pub mod commandline;
mod cursor;
mod enumeration;
pub mod history;
pub mod junkyard;
mod mark;
mod mode;
mod modification;
mod navigation;
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
mod viewport;

const SORT: fn(&BufferLine, &BufferLine) -> Ordering = |a, b| {
    a.content
        .to_ascii_uppercase()
        .cmp(&b.content.to_ascii_uppercase())
};

#[tracing::instrument(skip(model))]
pub fn update_model(model: &mut Model, envelope: &Envelope) -> Vec<Action> {
    match &envelope.sequence {
        KeySequence::Completed(_) => model.key_sequence.clear(),
        KeySequence::Changed(sequence) => sequence.clone_into(&mut model.key_sequence),
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
        Message::ExecuteKeySequence(_) => create_or_extend_command_stack(model, message),
        Message::ExecuteRegister(register) => replay_register(&mut model.register, register),
        Message::LeaveCommandMode => leave_commandline(model),
        Message::NavigateToMark(char) => navigate_to_mark(char, model),
        Message::NavigateToParent => navigate_to_parent(model),
        Message::NavigateToPath(path) => navigate_to_path(model, path),
        Message::NavigateToPathAsPreview(path) => navigate_to_path_as_preview(model, path),
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
        Message::ReplayMacro(char) => replay_macro_register(&mut model.register, char),
        Message::SetMark(char) => add_mark(model, *char),
        Message::StartMacro(identifier) => set_recording_in_commandline(model, *identifier),
        Message::StopMacro => set_mode_in_commandline(model),
        Message::ToggleQuickFix => toggle_selected_to_qfix(model),
        Message::Quit => vec![Action::Quit(None)],
        Message::YankPathToClipboard => copy_current_selected_path_to_clipboard(model),
        Message::YankToJunkYard(repeat) => yank_to_junkyard(model, repeat),
    }
}

#[tracing::instrument(skip(model, msg))]
pub fn update_with_buffer_message(model: &mut Model, msg: &BufferMessage) -> Vec<Action> {
    match msg {
        BufferMessage::ChangeMode(from, to) => change_mode(model, from, to),
        BufferMessage::Modification(repeat, modification) => match model.mode {
            Mode::Command(_) => update_commandline_on_modification(model, repeat, modification),
            Mode::Insert | Mode::Normal => modify_buffer(model, repeat, modification),
            Mode::Navigation => Vec::new(),
        },
        BufferMessage::MoveCursor(rpt, mtn) => match &model.mode {
            Mode::Command(_) => update_commandline(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => move_cursor(model, rpt, mtn),
        },
        BufferMessage::MoveViewPort(mtn) => match model.mode {
            Mode::Command(_) => update_commandline(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => move_viewport(model, mtn),
        },
        BufferMessage::SaveBuffer => persist_path_changes(model),

        BufferMessage::RemoveLine(_)
        | BufferMessage::ResetCursor
        | BufferMessage::SetContent(_)
        | BufferMessage::SetCursorToLineContent(_)
        | BufferMessage::SortContent(_) => unreachable!(),
    }
}

pub fn update_current(model: &mut Model, message: &BufferMessage) {
    let buffer = &mut model.files.current.buffer;
    let layout = &model.layout.current;

    set_viewport_dimensions(&mut buffer.view_port, layout);
    update_buffer(&model.mode, buffer, message);
}
