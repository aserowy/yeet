use std::{cmp::Ordering, path::Path};

use tokio_util::sync::CancellationToken;
use yeet_buffer::{
    message::BufferMessage,
    model::{ansi::Ansi, BufferLine, Mode, TextBuffer},
    update::update_buffer,
};
use yeet_keymap::message::{KeySequence, KeymapMessage, PrintContent};

use crate::{
    action::Action,
    event::{Envelope, Message, Preview},
    layout::AppLayout,
    model::{
        history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, register::Register,
        Buffer, CommandLine, CurrentTask, FileTreeBuffer, FileTreeBufferSection,
        FileTreeBufferSectionBuffer, Model,
    },
    settings::Settings,
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
    settings::update(model);

    let keymaps: Vec<_> = envelope.clone_keymap_messages();
    start_register_scope(&model.mode, &mut model.register, &keymaps);

    let sequence = envelope.sequence.clone();

    let actions = envelope
        .messages
        .into_iter()
        .flat_map(|message| {
            update_with_message(
                &mut model.commandline,
                &mut model.history,
                &mut model.junk,
                &mut model.layout,
                &mut model.marks,
                &mut model.qfix,
                &mut model.register,
                &mut model.remaining_keysequence,
                &mut model.settings,
                &mut model.mode,
                &mut model.buffer,
                message,
            )
        })
        .collect();

    finish_register_scope(&model.mode, &mut model.register, &sequence, &keymaps);

    actions
}

#[tracing::instrument(skip(commandline, buffer, layout, register))]
fn update_with_message(
    commandline: &mut CommandLine,
    history: &History,
    junk: &mut JunkYard,
    layout: &AppLayout,
    marks: &mut Marks,
    qfix: &mut QuickFix,
    register: &mut Register,
    remaining_keysequence: &mut Option<String>,
    settings: &Settings,
    mode: &mut Mode,
    buffer: &mut Buffer,
    message: Message,
) -> Vec<Action> {
    let buffer = match buffer {
        Buffer::FileTree(it) => it,
        Buffer::Text(_) => todo!(),
    };

    match message {
        Message::EnumerationChanged(path, contents, selection) => {
            update_on_enumeration_change(marks, qfix, mode, buffer, &path, &contents, &selection)
        }
        Message::EnumerationFinished(path, contents, selection) => update_on_enumeration_finished(
            history, marks, qfix, mode, buffer, &path, &contents, &selection,
        ),
        Message::Error(error) => {
            print_in_commandline(commandline, mode, &[PrintContent::Error(error.to_string())])
        }
        Message::FdResult(paths) => qfix::add(qfix, buffer, paths),
        Message::Keymap(msg) => update_with_keymap_message(
            commandline,
            history,
            junk,
            layout,
            marks,
            qfix,
            register,
            remaining_keysequence,
            settings,
            mode,
            buffer,
            &msg,
        ),
        Message::PathRemoved(path) => remove_path(history, junk, mode, buffer, &path),
        Message::PathsAdded(paths) => add_paths(history, marks, qfix, mode, buffer, &paths)
            .into_iter()
            .chain(add_to_junkyard(model, &paths).into_iter())
            .collect(),
        Message::PreviewLoaded(content) => update_preview(history, layout, mode, buffer, content),
        Message::Rerender => Vec::new(),
        Message::Resize(x, y) => vec![Action::Resize(x, y)],
        Message::TaskStarted(identifier, cancellation) => {
            add_current_task(model, identifier, cancellation)
        }
        Message::TaskEnded(identifier) => remove_current_task(model, identifier),
        Message::ZoxideResult(path) => navigate_to_path(model, path.as_ref()),
    }
}

#[tracing::instrument(skip(commandline, buffer, layout, msg, register))]
pub fn update_with_keymap_message(
    commandline: &mut CommandLine,
    history: &History,
    junk: &mut JunkYard,
    layout: &AppLayout,
    marks: &mut Marks,
    qfix: &mut QuickFix,
    register: &mut Register,
    remaining_keysequence: &mut Option<String>,
    settings: &Settings,
    mode: &mut Mode,
    buffer: &mut FileTreeBuffer,
    msg: &KeymapMessage,
) -> Vec<Action> {
    match msg {
        KeymapMessage::Buffer(msg) => update_with_buffer_message(
            commandline,
            history,
            junk,
            layout,
            marks,
            qfix,
            register,
            mode,
            buffer,
            msg,
        ),
        KeymapMessage::ClearSearchHighlight => clear_search(buffer),
        KeymapMessage::DeleteMarks(mrks) => delete_mark(marks, buffer, mrks),
        KeymapMessage::ExecuteCommand => update_commandline_on_execute(model),
        KeymapMessage::ExecuteCommandString(command) => command::execute(command, model),
        KeymapMessage::ExecuteKeySequence(key_sequence) => {
            remaining_keysequence.replace(key_sequence.clone());
            Vec::new()
        }
        KeymapMessage::ExecuteRegister(register) => replay_register(&mut model.register, register),
        KeymapMessage::LeaveCommandMode => leave_commandline(model),
        KeymapMessage::NavigateToMark(char) => navigate_to_mark(char, model),
        KeymapMessage::NavigateToParent => navigate_to_parent(model),
        KeymapMessage::NavigateToPath(path) => navigate_to_path(model, path),
        KeymapMessage::NavigateToPathAsPreview(path) => navigate_to_path_as_preview(model, path),
        KeymapMessage::NavigateToSelected => navigate_to_selected(model),
        KeymapMessage::OpenSelected => open_selected(settings, mode, buffer),
        KeymapMessage::PasteFromJunkYard(entry_id) => paste_to_junkyard(junk, buffer, entry_id),
        KeymapMessage::Print(content) => print_in_commandline(commandline, mode, content),
        KeymapMessage::ReplayMacro(char) => replay_macro_register(&mut model.register, char),
        KeymapMessage::SetMark(char) => add_mark(marks, buffer, *char),
        KeymapMessage::StartMacro(identifier) => set_recording_in_commandline(model, *identifier),
        KeymapMessage::StopMacro => set_mode_in_commandline(model),
        KeymapMessage::ToggleQuickFix => toggle_selected_to_qfix(qfix, buffer),
        KeymapMessage::Quit(mode) => vec![Action::Quit(mode.clone(), None)],
        KeymapMessage::YankPathToClipboard => {
            copy_current_selected_path_to_clipboard(register, buffer)
        }
        KeymapMessage::YankToJunkYard(repeat) => yank_to_junkyard(junk, buffer, repeat),
    }
}

#[tracing::instrument(skip(commandline, buffer, layout, msg, register))]
pub fn update_with_buffer_message(
    commandline: &mut CommandLine,
    history: &History,
    junk: &mut JunkYard,
    layout: &AppLayout,
    marks: &mut Marks,
    qfix: &mut QuickFix,
    register: &mut Register,
    mode: &mut Mode,
    buffer: &mut FileTreeBuffer,
    msg: &BufferMessage,
) -> Vec<Action> {
    match msg {
        BufferMessage::ChangeMode(from, to) => change_mode(model, from, to),
        BufferMessage::Modification(repeat, modification) => match mode {
            Mode::Command(_) => update_commandline_on_modification(model, repeat, modification),
            Mode::Insert | Mode::Normal => {
                modify_buffer(layout, mode, buffer, repeat, modification)
            }
            Mode::Navigation => Vec::new(),
        },
        BufferMessage::MoveCursor(rpt, mtn) => match mode {
            Mode::Command(_) => update_commandline(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                move_cursor(history, register, layout, mode, buffer, rpt, mtn)
            }
        },
        BufferMessage::MoveViewPort(mtn) => match mode {
            Mode::Command(_) => update_commandline(model, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                move_viewport(history, layout, mode, buffer, mtn)
            }
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

pub fn update_current(
    layout: &AppLayout,
    mode: &Mode,
    buffer: &mut FileTreeBuffer,
    message: &BufferMessage,
) {
    let viewport = &mut buffer.current_vp;
    let layout = &layout.current;

    set_viewport_dimensions(viewport, layout);

    update_buffer(
        viewport,
        &mut buffer.current_cursor,
        mode,
        &mut buffer.current.buffer,
        message,
    );
}

pub fn update_preview(
    history: &History,
    layout: &AppLayout,
    mode: &Mode,
    buffer: &mut FileTreeBuffer,
    content: Preview,
) -> Vec<Action> {
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

            buffer_type(
                history,
                layout,
                mode,
                buffer,
                &FileTreeBufferSection::Preview,
                &path,
                content,
            );
        }
        Preview::Image(path, protocol) => {
            buffer.preview = FileTreeBufferSectionBuffer::Image(path, protocol)
        }
        Preview::None(_) => buffer.preview = FileTreeBufferSectionBuffer::None,
    };
    Vec::new()
}

pub fn buffer_type(
    history: &History,
    layout: &AppLayout,
    mode: &Mode,
    buffer: &mut FileTreeBuffer,
    section: &FileTreeBufferSection,
    path: &Path,
    content: Vec<BufferLine>,
) {
    let mut text_buffer = TextBuffer::default();

    let (viewport, cursor, layout) = match section {
        FileTreeBufferSection::Parent => (
            &mut buffer.parent_vp,
            &mut buffer.parent_cursor,
            &layout.parent,
        ),
        FileTreeBufferSection::Preview => (
            &mut buffer.preview_vp,
            &mut buffer.preview_cursor,
            &layout.preview,
        ),
        FileTreeBufferSection::Current => unreachable!(),
    };

    update_buffer(
        viewport,
        cursor,
        mode,
        &mut text_buffer,
        &BufferMessage::SetContent(content.to_vec()),
    );

    viewport::set_viewport_dimensions(viewport, layout);
    update_buffer(
        viewport,
        cursor,
        mode,
        &mut text_buffer,
        &BufferMessage::ResetCursor,
    );

    if let Some(cursor) = cursor {
        cursor.hide_cursor_line = true;
    }

    if path.is_dir() {
        cursor::set_cursor_index_with_history(
            history,
            viewport,
            cursor,
            mode,
            &mut text_buffer,
            path,
        );
    }

    let buffer_type = FileTreeBufferSectionBuffer::Text(path.to_path_buf(), text_buffer);
    match section {
        FileTreeBufferSection::Parent => buffer.parent = buffer_type,
        FileTreeBufferSection::Preview => buffer.preview = buffer_type,
        FileTreeBufferSection::Current => unreachable!(),
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
        match next_id.cmp(&id) {
            Ordering::Equal => next_id += 1,
            Ordering::Greater => break,
            Ordering::Less => {}
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
