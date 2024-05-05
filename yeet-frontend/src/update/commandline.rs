use ratatui::style::Color;
use yeet_buffer::{
    message::{BufferMessage, CursorDirection, Search, TextModification},
    model::{BufferLine, CommandMode, Mode, SearchDirection, StylePartial, StylePartialSpan},
    update::update_buffer,
};
use yeet_keymap::message::{Message, PrintContent};

use crate::{
    action::Action,
    model::{register::RegisterScope, Model},
    update::search::{clear_search, search_in_buffers},
};

use super::set_viewport_dimensions;

pub fn update_commandline(model: &mut Model, message: Option<&BufferMessage>) -> Vec<Action> {
    let command_mode = match &model.mode {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => return Vec::new(),
    };

    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

    if let Some(message) = message {
        match command_mode {
            CommandMode::Command | CommandMode::Search(_) => {
                update_buffer(&model.mode, buffer, message);
            }
            CommandMode::PrintMultiline => {}
        }
    }

    Vec::new()
}

pub fn update_commandline_on_modification(
    model: &mut Model,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let command_mode = match &model.mode {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => return Vec::new(),
    };

    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

    match command_mode {
        CommandMode::Command | CommandMode::Search(_) => {
            let mut actions = Vec::new();
            if let &TextModification::DeleteMotion(_, CursorDirection::Left) = modification {
                if let Some(line) = buffer.lines.last() {
                    if line.content.is_empty() {
                        actions.push(Action::EmitMessages(vec![Message::Buffer(
                            BufferMessage::ChangeMode(
                                model.mode.clone(),
                                get_mode_after_command(&model.mode_before),
                            ),
                        )]));
                    }
                }
            };

            update_buffer(
                &model.mode,
                buffer,
                &BufferMessage::Modification(*repeat, modification.clone()),
            );

            actions
        }
        CommandMode::PrintMultiline => {
            let mut messages = Vec::new();
            if let TextModification::Insert(cnt) = modification {
                let action = if matches!(cnt.as_str(), ":" | "/" | "?") {
                    model.mode = Mode::Command(match cnt.as_str() {
                        ":" => CommandMode::Command,
                        "/" => CommandMode::Search(SearchDirection::Down),
                        "?" => CommandMode::Search(SearchDirection::Up),
                        _ => unreachable!(),
                    });

                    let bufferline = BufferLine {
                        prefix: Some(cnt.to_string()),
                        ..Default::default()
                    };

                    buffer.lines.pop();
                    buffer.lines.push(bufferline);

                    Message::Rerender
                } else {
                    update_buffer(&model.mode, buffer, &BufferMessage::SetContent(vec![]));

                    Message::Buffer(BufferMessage::ChangeMode(
                        model.mode.clone(),
                        get_mode_after_command(&model.mode_before),
                    ))
                };

                messages.push(Action::EmitMessages(vec![action]));
            }

            messages
        }
    }
}

pub fn update_commandline_on_mode_change(model: &mut Model) -> Vec<Action> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

    let command_mode = match &model.mode {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => {
            let from_command = model
                .mode_before
                .as_ref()
                .is_some_and(|mode| mode.is_command());

            if from_command {
                update_buffer(&model.mode, buffer, &BufferMessage::SetContent(vec![]));
            }
            return Vec::new();
        }
    };

    match command_mode {
        CommandMode::Command | CommandMode::Search(_) => {
            update_buffer(&model.mode, buffer, &BufferMessage::ResetCursor);

            let prefix = match &command_mode {
                CommandMode::Command => Some(":".to_string()),
                CommandMode::Search(SearchDirection::Up) => Some("?".to_string()),
                CommandMode::Search(SearchDirection::Down) => Some("/".to_string()),
                CommandMode::PrintMultiline => unreachable!(),
            };

            let bufferline = BufferLine {
                prefix,
                ..Default::default()
            };

            update_buffer(
                &model.mode,
                buffer,
                &BufferMessage::SetContent(vec![bufferline]),
            );
        }
        CommandMode::PrintMultiline => {}
    };

    Vec::new()
}

pub fn update_commandline_on_execute(model: &mut Model) -> Vec<Action> {
    let command_mode = match &model.mode {
        Mode::Command(it) => it,
        Mode::Insert | Mode::Navigation | Mode::Normal => return Vec::new(),
    };

    let messages = match command_mode {
        CommandMode::Command => {
            if let Some(cmd) = model.commandline.buffer.lines.last() {
                // TODO: add command history and show previous command not current (this enables g: as well)
                model.register.command = Some(cmd.content.clone());

                vec![Message::ExecuteCommandString(cmd.content.clone())]
            } else {
                Vec::new()
            }
        }
        CommandMode::PrintMultiline => {
            vec![Message::Buffer(BufferMessage::ChangeMode(
                model.mode.clone(),
                get_mode_after_command(&model.mode_before),
            ))]
        }
        CommandMode::Search(direction) => {
            model.register.searched = model
                .commandline
                .buffer
                .lines
                .last()
                .map(|bl| (direction.clone(), bl.content.clone()));

            if model.register.searched.is_none() {
                clear_search(model);
            }

            vec![
                Message::Buffer(BufferMessage::ChangeMode(
                    model.mode.clone(),
                    get_mode_after_command(&model.mode_before),
                )),
                Message::Buffer(BufferMessage::MoveCursor(
                    1,
                    CursorDirection::Search(Search::Next),
                )),
            ]
        }
    };

    update_buffer(
        &model.mode,
        &mut model.commandline.buffer,
        &BufferMessage::SetContent(vec![]),
    );

    vec![Action::EmitMessages(messages)]
}

pub fn leave_commandline(model: &mut Model) -> Vec<Action> {
    if matches!(model.mode, Mode::Command(CommandMode::Search(_))) {
        let content = model.register.get(&'/');
        search_in_buffers(model, content);
    }

    update_buffer(
        &model.mode,
        &mut model.commandline.buffer,
        &BufferMessage::SetContent(vec![]),
    );

    vec![Action::EmitMessages(vec![Message::Buffer(
        BufferMessage::ChangeMode(
            model.mode.clone(),
            get_mode_after_command(&model.mode_before),
        ),
    )])]
}

// TODO: buffer messages till command mode left
pub fn print_in_commandline(model: &mut Model, content: &[PrintContent]) -> Vec<Action> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

    commandline.buffer.lines = content
        .iter()
        .map(|content| match content {
            PrintContent::Default(cntnt) => BufferLine {
                content: cntnt.to_string(),
                ..Default::default()
            },
            PrintContent::Error(cntnt) => {
                let cntnt_len = cntnt.chars().count();
                BufferLine {
                    content: cntnt.to_string(),
                    style: vec![StylePartialSpan {
                        end: cntnt_len,
                        style: StylePartial::Foreground(Color::Red),
                        ..Default::default()
                    }],
                    ..Default::default()
                }
            }
            PrintContent::Information(cntnt) => {
                let cntnt_len = cntnt.chars().count();
                BufferLine {
                    content: cntnt.to_string(),
                    style: vec![StylePartialSpan {
                        end: cntnt_len,
                        style: StylePartial::Foreground(Color::LightGreen),
                        ..Default::default()
                    }],
                    ..Default::default()
                }
            }
        })
        .collect();

    let actions = if commandline.buffer.lines.len() > 1 {
        let content = "Press ENTER or type command to continue";
        commandline.buffer.lines.push(BufferLine {
            content: content.to_string(),
            style: vec![StylePartialSpan {
                end: content.chars().count(),
                style: StylePartial::Foreground(Color::LightBlue),
                ..Default::default()
            }],
            ..Default::default()
        });

        if model.mode.is_command() {
            model.mode = Mode::Command(CommandMode::PrintMultiline);
        }

        vec![Action::EmitMessages(vec![Message::Buffer(
            BufferMessage::ChangeMode(
                model.mode.clone(),
                Mode::Command(CommandMode::PrintMultiline),
            ),
        )])]
    } else {
        Vec::new()
    };

    update_buffer(
        &model.mode,
        &mut commandline.buffer,
        &BufferMessage::MoveCursor(1, CursorDirection::Bottom),
    );
    update_buffer(
        &model.mode,
        &mut commandline.buffer,
        &BufferMessage::MoveCursor(1, CursorDirection::LineEnd),
    );

    actions
}

pub fn height(model: &Model, messages: &Vec<Message>) -> u16 {
    let lines_len = model.commandline.buffer.lines.len();
    let mut height = if lines_len == 0 { 1 } else { lines_len as u16 };
    for message in messages {
        if let Message::Print(content) = message {
            if content.len() > 1 {
                height = content.len() as u16 + 1;
            }
        }
    }
    height
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

pub fn set_content_status(model: &mut Model) {
    if let Some(RegisterScope::Macro(identifier)) = &model.register.resolve_macro() {
        set_recording_in_commandline(model, *identifier);
    } else {
        set_mode_in_commandline(model);
    };
}

pub fn set_recording_in_commandline(model: &mut Model, identifier: char) -> Vec<Action> {
    let content = format!("recording @{}", identifier);
    print_in_commandline(model, &[PrintContent::Default(content)]);
    Vec::new()
}

pub fn set_mode_in_commandline(model: &mut Model) -> Vec<Action> {
    let content = format!("--{}--", model.mode.to_string().to_uppercase());
    print_in_commandline(model, &[PrintContent::Default(content)]);
    Vec::new()
}
