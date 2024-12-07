use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search, TextModification},
    model::{ansi::Ansi, BufferLine, CommandMode, Mode, SearchDirection},
    update::update_buffer,
};
use yeet_keymap::message::{KeymapMessage, PrintContent};

use crate::{
    action::{self, Action},
    event::Message,
    model::{register::Register, CommandLine, FileTreeBuffer, ModeState},
    update::{
        register::get_register,
        search::{self, search_in_buffers},
    },
};

use super::viewport;

pub fn update(
    commandline: &mut CommandLine,
    mode: &Mode,
    message: Option<&BufferMessage>,
) -> Vec<Action> {
    let command_mode = match mode {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => return Vec::new(),
    };

    let buffer = &mut commandline.buffer;
    let cursor = &mut commandline.cursor;
    let viewport = &mut commandline.viewport;

    viewport::set_dimensions(viewport, &commandline.layout.buffer);

    if let Some(message) = message {
        match command_mode {
            CommandMode::Command | CommandMode::Search(_) => {
                update_buffer(viewport, cursor, mode, buffer, message);
            }
            CommandMode::PrintMultiline => {}
        }
    }

    Vec::new()
}

pub fn modify(
    commandline: &mut CommandLine,
    modes: &mut ModeState,
    buffer: &mut FileTreeBuffer,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let command_mode = match &modes.current {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => return Vec::new(),
    };

    let text_buffer = &mut commandline.buffer;
    let cursor = &mut commandline.cursor;
    let viewport = &mut commandline.viewport;

    viewport::set_dimensions(viewport, &commandline.layout.buffer);

    match command_mode {
        CommandMode::Command | CommandMode::Search(_) => {
            let mut actions = Vec::new();
            if let &TextModification::DeleteMotion(_, CursorDirection::Left) = modification {
                if let Some(line) = text_buffer.lines.last() {
                    if line.content.is_empty() {
                        actions.push(action::emit_keymap(KeymapMessage::Buffer(
                            BufferMessage::ChangeMode(
                                modes.current.clone(),
                                get_mode_after_command(&modes.previous),
                            ),
                        )));
                    }
                }
            };

            update_buffer(
                viewport,
                cursor,
                &modes.current,
                text_buffer,
                &BufferMessage::Modification(*repeat, modification.clone()),
            );

            if matches!(modes.current, Mode::Command(CommandMode::Search(_))) {
                let term = commandline
                    .buffer
                    .lines
                    .last()
                    .map(|bl| bl.content.to_stripped_string());

                search_in_buffers(buffer, term);
            }

            actions
        }
        CommandMode::PrintMultiline => {
            let mut messages = Vec::new();
            if let TextModification::Insert(cnt) = modification {
                let action = if matches!(cnt.as_str(), ":" | "/" | "?") {
                    modes.current = Mode::Command(match cnt.as_str() {
                        ":" => CommandMode::Command,
                        "/" => CommandMode::Search(SearchDirection::Down),
                        "?" => CommandMode::Search(SearchDirection::Up),
                        _ => unreachable!(),
                    });

                    let bufferline = BufferLine {
                        prefix: Some(cnt.to_string()),
                        ..Default::default()
                    };

                    text_buffer.lines.pop();
                    text_buffer.lines.push(bufferline);

                    Message::Rerender
                } else {
                    update_buffer(
                        viewport,
                        cursor,
                        &modes.current,
                        text_buffer,
                        &BufferMessage::SetContent(vec![]),
                    );

                    Message::Keymap(KeymapMessage::Buffer(BufferMessage::ChangeMode(
                        modes.current.clone(),
                        get_mode_after_command(&modes.previous),
                    )))
                };

                messages.push(Action::EmitMessages(vec![action]));
            }

            messages
        }
    }
}

pub fn update_on_execute(
    commandline: &mut CommandLine,
    register: &mut Register,
    modes: &mut ModeState,
    buffer: &mut FileTreeBuffer,
) -> Vec<Action> {
    let command_mode = match &modes.current {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => return Vec::new(),
    };

    let messages = match command_mode {
        CommandMode::Command => {
            if let Some(cmd) = commandline.buffer.lines.last() {
                // TODO: add command history and show previous command not current (this enables g: as well)
                register.command = Some(cmd.content.to_stripped_string());

                vec![Message::Keymap(KeymapMessage::ExecuteCommandString(
                    cmd.content.to_stripped_string(),
                ))]
            } else {
                Vec::new()
            }
        }
        CommandMode::PrintMultiline => {
            vec![Message::Keymap(KeymapMessage::Buffer(
                BufferMessage::ChangeMode(
                    modes.current.clone(),
                    get_mode_after_command(&modes.previous),
                ),
            ))]
        }
        CommandMode::Search(direction) => {
            register.searched = commandline
                .buffer
                .lines
                .last()
                .map(|bl| (direction.clone(), bl.content.to_stripped_string()));

            if register.searched.is_none() {
                search::clear(buffer);
            }

            vec![
                Message::Keymap(KeymapMessage::Buffer(BufferMessage::ChangeMode(
                    modes.current.clone(),
                    get_mode_after_command(&modes.previous),
                ))),
                Message::Keymap(KeymapMessage::Buffer(BufferMessage::MoveCursor(
                    1,
                    CursorDirection::Search(Search::Next),
                ))),
            ]
        }
    };

    update_buffer(
        &mut commandline.viewport,
        &mut commandline.cursor,
        &modes.current,
        &mut commandline.buffer,
        &BufferMessage::SetContent(vec![]),
    );

    vec![Action::EmitMessages(messages)]
}

pub fn leave(
    commandline: &mut CommandLine,
    register: &mut Register,
    modes: &ModeState,
    buffer: &mut FileTreeBuffer,
) -> Vec<Action> {
    if matches!(modes.current, Mode::Command(CommandMode::Search(_))) {
        let content = get_register(register, &'/');
        search_in_buffers(buffer, content);
    }

    update_buffer(
        &mut commandline.viewport,
        &mut commandline.cursor,
        &modes.current,
        &mut commandline.buffer,
        &BufferMessage::SetContent(vec![]),
    );

    vec![action::emit_keymap(KeymapMessage::Buffer(
        BufferMessage::ChangeMode(
            modes.current.clone(),
            get_mode_after_command(&modes.previous),
        ),
    ))]
}

// TODO: buffer messages till command mode left
pub fn print(
    commandline: &mut CommandLine,
    modes: &mut ModeState,
    content: &[PrintContent],
) -> Vec<Action> {
    let viewport = &mut commandline.viewport;

    viewport::set_dimensions(viewport, &commandline.layout.buffer);

    commandline.buffer.lines = content
        .iter()
        .map(|content| match content {
            PrintContent::Default(cntnt) => BufferLine {
                content: Ansi::new(&cntnt.to_string()),
                ..Default::default()
            },
            PrintContent::Error(cntnt) => BufferLine {
                content: Ansi::new(&format!("\x1b[31m{}\x1b[39m", cntnt)),
                ..Default::default()
            },
            PrintContent::Information(cntnt) => BufferLine {
                content: Ansi::new(&format!("\x1b[92m{}\x1b[39m", cntnt)),
                ..Default::default()
            },
        })
        .collect();

    let actions = if commandline.buffer.lines.len() > 1 {
        let content = "Press ENTER or type command to continue";
        commandline.buffer.lines.push(BufferLine {
            content: Ansi::new(&format!("\x1b[94m{}\x1b[39m", content)),
            ..Default::default()
        });

        if modes.current.is_command() {
            modes.current = Mode::Command(CommandMode::PrintMultiline);
        }

        vec![action::emit_keymap(KeymapMessage::Buffer(
            BufferMessage::ChangeMode(
                modes.current.clone(),
                Mode::Command(CommandMode::PrintMultiline),
            ),
        ))]
    } else {
        Vec::new()
    };

    update_buffer(
        &mut commandline.viewport,
        &mut commandline.cursor,
        &modes.current,
        &mut commandline.buffer,
        &BufferMessage::MoveCursor(1, CursorDirection::Bottom),
    );
    update_buffer(
        &mut commandline.viewport,
        &mut commandline.cursor,
        &modes.current,
        &mut commandline.buffer,
        &BufferMessage::MoveCursor(1, CursorDirection::LineEnd),
    );

    actions
}

fn get_mode_after_command(mode_before: &Option<Mode>) -> Mode {
    if let Some(mode) = mode_before {
        match mode {
            Mode::Command(_) => unreachable!(),
            Mode::Insert | Mode::Normal => Mode::Normal,
            Mode::Navigation => Mode::Navigation,
        }
    } else {
        Mode::default()
    }
}
