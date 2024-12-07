use std::{cmp::Ordering, path::Path};

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
        history::History, App, Buffer, CommandLine, FileTreeBuffer, FileTreeBufferSection,
        FileTreeBufferSectionBuffer, Model, State,
    },
    settings::Settings,
};

mod command;
pub mod commandline;
mod cursor;
mod enumeration;
pub mod history;
pub mod junkyard;
mod mark;
mod mode;
mod modify;
mod navigate;
mod open;
mod path;
mod qfix;
mod register;
mod save;
mod search;
mod selection;
mod settings;
mod sign;
mod task;
pub mod viewport;

const SORT: fn(&BufferLine, &BufferLine) -> Ordering = |a, b| {
    a.content
        .to_stripped_string()
        .to_ascii_uppercase()
        .cmp(&b.content.to_stripped_string().to_ascii_uppercase())
};

#[tracing::instrument(skip(model))]
pub fn model(model: &mut Model, envelope: Envelope) -> Vec<Action> {
    match &envelope.sequence {
        KeySequence::Completed(_) => model.app.commandline.key_sequence.clear(),
        KeySequence::Changed(sequence) => {
            sequence.clone_into(&mut model.app.commandline.key_sequence)
        }
        KeySequence::None => {}
    };

    commandline::update(&mut model.app.commandline, &model.state.modes.current, None);
    settings::update(model);

    let keymaps: Vec<_> = envelope.clone_keymap_messages();
    register::start_scope(
        &model.state.modes.current,
        &mut model.state.register,
        &keymaps,
    );

    let sequence = envelope.sequence.clone();

    let actions = envelope
        .messages
        .into_iter()
        .flat_map(|message| {
            update_with_message(&mut model.app, &mut model.state, &model.settings, message)
        })
        .collect();

    register::finish_scope(
        &model.state.modes.current,
        &mut model.state.register,
        &sequence,
        &keymaps,
    );

    actions
}

#[tracing::instrument(skip_all)]
fn update_with_message(
    app: &mut App,
    state: &mut State,
    settings: &Settings,
    message: Message,
) -> Vec<Action> {
    let buffer = match &mut app.buffer {
        Buffer::FileTree(it) => it,
        Buffer::_Text(_) => todo!(),
    };

    match message {
        Message::EnumerationChanged(path, contents, selection) => {
            enumeration::change(state, buffer, &path, &contents, &selection)
        }
        Message::EnumerationFinished(path, contents, selection) => {
            enumeration::finish(state, buffer, &path, &contents, &selection)
        }
        Message::Error(error) => commandline::print(
            &mut app.commandline,
            &mut state.modes,
            &[PrintContent::Error(error.to_string())],
        ),
        Message::FdResult(paths) => qfix::add(&mut state.qfix, buffer, paths),
        Message::Keymap(msg) => update_with_keymap_message(
            state,
            settings,
            &mut app.commandline,
            &app.layout,
            buffer,
            &msg,
        ),
        Message::PathRemoved(path) => path::remove(
            &state.history,
            &mut state.junk,
            &state.modes.current,
            buffer,
            &path,
        ),
        Message::PathsAdded(paths) => path::add(
            &state.history,
            &state.marks,
            &state.qfix,
            &state.modes.current,
            buffer,
            &paths,
        )
        .into_iter()
        .chain(junkyard::add(&mut state.junk, &paths).into_iter())
        .collect(),
        Message::PreviewLoaded(content) => update_preview(
            &state.history,
            &app.layout,
            &state.modes.current,
            buffer,
            content,
        ),
        Message::Rerender => Vec::new(),
        Message::Resize(x, y) => vec![Action::Resize(x, y)],
        Message::TaskStarted(id, cancellation) => task::add(&mut state.tasks, id, cancellation),
        Message::TaskEnded(id) => task::remove(&mut state.tasks, id),
        Message::ZoxideResult(path) => navigate::path(&state.history, buffer, path.as_ref()),
    }
}

#[tracing::instrument(skip_all)]
pub fn update_with_keymap_message(
    state: &mut State,
    settings: &Settings,
    commandline: &mut CommandLine,
    layout: &AppLayout,
    buffer: &mut FileTreeBuffer,
    msg: &KeymapMessage,
) -> Vec<Action> {
    match msg {
        KeymapMessage::Buffer(msg) => {
            update_with_buffer_message(state, commandline, layout, buffer, msg)
        }
        KeymapMessage::ClearSearchHighlight => search::clear(buffer),
        KeymapMessage::DeleteMarks(mrks) => mark::delete(&mut state.marks, buffer, mrks),
        KeymapMessage::ExecuteCommand => commandline::update_on_execute(
            commandline,
            &mut state.register,
            &mut state.modes,
            buffer,
        ),
        KeymapMessage::ExecuteCommandString(command) => command::execute(state, buffer, command),
        KeymapMessage::ExecuteKeySequence(key_sequence) => {
            state.remaining_keysequence.replace(key_sequence.clone());
            Vec::new()
        }
        KeymapMessage::ExecuteRegister(rgstr) => register::replay(&mut state.register, rgstr),
        KeymapMessage::LeaveCommandMode => {
            commandline::leave(commandline, &mut state.register, &state.modes, buffer)
        }
        KeymapMessage::NavigateToMark(char) => {
            navigate::mark(&state.history, &state.marks, buffer, char)
        }
        KeymapMessage::NavigateToParent => navigate::parent(buffer),
        KeymapMessage::NavigateToPath(path) => navigate::path(&state.history, buffer, path),
        KeymapMessage::NavigateToPathAsPreview(path) => {
            navigate::path_as_preview(&state.history, buffer, path)
        }
        KeymapMessage::NavigateToSelected => navigate::selected(&mut state.history, buffer),
        KeymapMessage::OpenSelected => open::selected(settings, &state.modes.current, buffer),
        KeymapMessage::PasteFromJunkYard(entry_id) => {
            junkyard::paste(&state.junk, buffer, entry_id)
        }
        KeymapMessage::Print(content) => commandline::print(commandline, &mut state.modes, content),
        KeymapMessage::ReplayMacro(char) => register::replay_macro(&mut state.register, char),
        KeymapMessage::SetMark(char) => mark::add(&mut state.marks, buffer, *char),
        KeymapMessage::StartMacro(identifier) => {
            mode::print_recording(commandline, &mut state.modes, *identifier)
        }
        KeymapMessage::StopMacro => mode::print_mode(commandline, &mut state.modes),
        KeymapMessage::ToggleQuickFix => qfix::toggle(&mut state.qfix, buffer),
        KeymapMessage::Quit(mode) => vec![Action::Quit(mode.clone(), None)],
        KeymapMessage::YankPathToClipboard => {
            selection::copy_to_clipboard(&mut state.register, buffer)
        }
        KeymapMessage::YankToJunkYard(repeat) => junkyard::yank(&mut state.junk, buffer, repeat),
    }
}

#[tracing::instrument(skip_all)]
pub fn update_with_buffer_message(
    state: &mut State,
    commandline: &mut CommandLine,
    layout: &AppLayout,
    buffer: &mut FileTreeBuffer,
    msg: &BufferMessage,
) -> Vec<Action> {
    match msg {
        BufferMessage::ChangeMode(from, to) => {
            mode::change(state, commandline, layout, buffer, from, to)
        }
        BufferMessage::Modification(repeat, modification) => match &mut state.modes.current {
            Mode::Command(_) => {
                commandline::modify(commandline, &mut state.modes, buffer, repeat, modification)
            }
            Mode::Insert | Mode::Normal => {
                modify::buffer(layout, &state.modes.current, buffer, repeat, modification)
            }
            Mode::Navigation => Vec::new(),
        },
        BufferMessage::MoveCursor(rpt, mtn) => match &mut state.modes.current {
            Mode::Command(_) => commandline::update(commandline, &state.modes.current, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                cursor::change(state, layout, buffer, rpt, mtn)
            }
        },
        BufferMessage::MoveViewPort(mtn) => match &state.modes.current {
            Mode::Command(_) => commandline::update(commandline, &state.modes.current, Some(msg)),
            Mode::Insert | Mode::Navigation | Mode::Normal => {
                viewport::change(&state.history, layout, &state.modes.current, buffer, mtn)
            }
        },
        BufferMessage::SaveBuffer => save::changes(&mut state.junk, &state.modes.current, buffer),

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

    viewport::set_dimensions(viewport, layout);

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

    viewport::set_dimensions(viewport, layout);
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
