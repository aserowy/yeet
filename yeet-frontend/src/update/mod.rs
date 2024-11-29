use std::{cmp::Ordering, path::Path};

use tokio_util::sync::CancellationToken;
use yeet_buffer::{
    message::BufferMessage,
    model::{ansi::Ansi, Buffer, BufferLine, Mode},
    update::update_buffer,
};
use yeet_keymap::message::{KeySequence, KeymapMessage, PrintContent};

use crate::{
    action::Action,
    event::{Envelope, Message, Preview},
    model::{BufferType, CurrentTask, Model, WindowType},
};

use self::{
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
mod qfix;
mod register;
mod save;
mod search;
mod selection;
mod settings;
mod sign;
pub mod viewport;

const SORT: fn(&BufferLine, &BufferLine) -> Ordering = |a, b| {
    a.content
        .to_stripped_string()
        .to_ascii_uppercase()
        .cmp(&b.content.to_stripped_string().to_ascii_uppercase())
};

#[tracing::instrument(skip(model))]
pub fn update_model(model: &mut Model, envelope: Envelope) -> Vec<Action> {
    match &envelope.sequence {
        KeySequence::Completed(_) => model.commandline.key_sequence.clear(),
        KeySequence::Changed(sequence) => sequence.clone_into(&mut model.commandline.key_sequence),
        KeySequence::None => {}
    };

    update_commandline(model, None);
    update_with_settings(model);

    let keymaps: Vec<_> = envelope.clone_keymap_messages();
    start_register_scope(&model.mode, &mut model.register, &keymaps);

    let sequence = envelope.sequence.clone();

    let actions = envelope
        .messages
        .into_iter()
        .flat_map(|message| update_with_message(model, message))
        .collect();

    finish_register_scope(&model.mode, &mut model.register, &sequence, &keymaps);

    actions
}

#[tracing::instrument(skip(model))]
fn update_with_message(model: &mut Model, message: Message) -> Vec<Action> {
    match message {
        Message::EnumerationChanged(path, contents, selection) => {
            update_on_enumeration_change(model, &path, &contents, &selection)
        }
        Message::EnumerationFinished(path, selection) => {
            update_on_enumeration_finished(model, &path, &selection)
        }
        Message::Error(error) => {
            print_in_commandline(model, &[PrintContent::Error(error.to_string())])
        }
        Message::FdResult(paths) => qfix::add(model, paths),
        Message::Keymap(msg) => update_with_keymap_message(model, &msg),
        Message::PathRemoved(path) => remove_path(model, &path),
        Message::PathsAdded(paths) => add_paths(model, &paths)
            .into_iter()
            .chain(add_to_junkyard(model, &paths).into_iter())
            .collect(),
        Message::PreviewLoaded(content) => update_preview(model, content),
        Message::Rerender => Vec::new(),
        Message::Resize(x, y) => vec![Action::Resize(x, y)],
        Message::TaskStarted(identifier, cancellation) => {
            add_current_task(model, identifier, cancellation)
        }
        Message::TaskEnded(identifier) => remove_current_task(model, identifier),
    }
}

#[tracing::instrument(skip(model, msg))]
pub fn update_with_keymap_message(model: &mut Model, msg: &KeymapMessage) -> Vec<Action> {
    match msg {
        KeymapMessage::Buffer(msg) => update_with_buffer_message(model, msg),
        KeymapMessage::ClearSearchHighlight => clear_search(model),
        KeymapMessage::DeleteMarks(marks) => delete_mark(model, marks),
        KeymapMessage::ExecuteCommand => update_commandline_on_execute(model),
        KeymapMessage::ExecuteCommandString(command) => command::execute(command, model),
        KeymapMessage::ExecuteKeySequence(key_sequence) => {
            super::set_remaining_keysequence(model, key_sequence)
        }
        KeymapMessage::ExecuteRegister(register) => replay_register(&mut model.register, register),
        KeymapMessage::LeaveCommandMode => leave_commandline(model),
        KeymapMessage::NavigateToMark(char) => navigate_to_mark(char, model),
        KeymapMessage::NavigateToParent => navigate_to_parent(model),
        KeymapMessage::NavigateToPath(path) => navigate_to_path(model, path),
        KeymapMessage::NavigateToPathAsPreview(path) => navigate_to_path_as_preview(model, path),
        KeymapMessage::NavigateToSelected => navigate_to_selected(model),
        KeymapMessage::OpenSelected => open_selected(model),
        KeymapMessage::PasteFromJunkYard(entry_id) => paste_to_junkyard(model, entry_id),
        KeymapMessage::Print(content) => print_in_commandline(model, content),
        KeymapMessage::ReplayMacro(char) => replay_macro_register(&mut model.register, char),
        KeymapMessage::SetMark(char) => add_mark(model, *char),
        KeymapMessage::StartMacro(identifier) => set_recording_in_commandline(model, *identifier),
        KeymapMessage::StopMacro => set_mode_in_commandline(model),
        KeymapMessage::ToggleQuickFix => toggle_selected_to_qfix(model),
        KeymapMessage::Quit(mode) => vec![Action::Quit(mode.clone(), None)],
        KeymapMessage::YankPathToClipboard => copy_current_selected_path_to_clipboard(model),
        KeymapMessage::YankToJunkYard(repeat) => yank_to_junkyard(model, repeat),
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
        | BufferMessage::SortContent(_)
        | BufferMessage::UpdateViewPortByCursor => unreachable!(),
    }
}

pub fn update_current(model: &mut Model, message: &BufferMessage) {
    let buffer = &mut model.files.current.buffer;
    let layout = &model.layout.current;

    set_viewport_dimensions(&mut buffer.view_port, layout);
    update_buffer(&model.mode, buffer, message);
}

pub fn update_preview(model: &mut Model, content: Preview) -> Vec<Action> {
    match content {
        Preview::Content(path, content) => {
            tracing::trace!("updating preview buffer: {:?}", path);

            let content = content
                .iter()
                .map(|s| BufferLine {
                    content: Ansi::new(s),
                    ..Default::default()
                })
                .collect();

            buffer_type(&WindowType::Preview, model, &path, content);
        }
        Preview::Image(path, protocol) => model.files.preview = BufferType::Image(path, protocol),
        Preview::None(_) => model.files.preview = BufferType::None,
    };
    Vec::new()
}

pub fn buffer_type(
    window_type: &WindowType,
    model: &mut Model,
    path: &Path,
    content: Vec<BufferLine>,
) {
    let mut buffer = Buffer::default();

    update_buffer(
        &model.mode,
        &mut buffer,
        &BufferMessage::SetContent(content.to_vec()),
    );

    let layout = match window_type {
        WindowType::Parent => &model.layout.parent,
        WindowType::Preview => &model.layout.preview,
        WindowType::Current => unreachable!(),
    };

    viewport::set_viewport_dimensions(&mut buffer.view_port, layout);
    update_buffer(&model.mode, &mut buffer, &BufferMessage::ResetCursor);

    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor_line = true;
    }

    if path.is_dir() {
        cursor::set_cursor_index_with_history(&model.mode, &model.history, &mut buffer, path);
    }

    let buffer_type = BufferType::Text(path.to_path_buf(), buffer);
    match window_type {
        WindowType::Parent => model.files.parent = buffer_type,
        WindowType::Preview => model.files.preview = buffer_type,
        WindowType::Current => unreachable!(),
    };
}

fn add_current_task(
    model: &mut Model,
    identifier: String,
    cancellation: CancellationToken,
) -> Vec<Action> {
    let id = next_id(model);

    if let Some(replaced_task) = model.current_tasks.insert(
        identifier.clone(),
        CurrentTask {
            token: cancellation,
            id,
            external_id: identifier,
        },
    ) {
        replaced_task.token.cancel();
    }
    Vec::new()
}

fn next_id(model: &mut Model) -> u16 {
    let mut next_id = if model.latest_task_id >= 9999 {
        1
    } else {
        model.latest_task_id + 1
    };

    let mut running_ids: Vec<u16> = model.current_tasks.values().map(|task| task.id).collect();
    running_ids.sort();

    for id in running_ids {
        if next_id == id {
            next_id += 1;
        } else if next_id > id {
            break;
        }
    }

    model.latest_task_id = next_id;

    next_id
}

fn remove_current_task(model: &mut Model, identifier: String) -> Vec<Action> {
    if let Some(task) = model.current_tasks.remove(&identifier) {
        task.token.cancel();
    }
    Vec::new()
}
