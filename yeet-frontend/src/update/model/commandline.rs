use ratatui::style::Color;
use yeet_buffer::{
    message::{BufferMessage, CursorDirection, SearchDirection, TextModification},
    model::{BufferLine, CommandMode, Mode, StylePartial, StylePartialSpan},
    update,
};
use yeet_keymap::message::{Message, PrintContent};

use crate::{
    action::Action,
    model::{CommandLineState, Model},
};

pub fn update(model: &mut Model, message: Option<&BufferMessage>) -> Vec<Action> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    super::set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

    match commandline.state {
        CommandLineState::Default => {
            let mut actions = vec![
                Action::SkipRender,
                Action::EmitMessages(vec![Message::Rerender]),
            ];

            if let Some(message) = message {
                match message {
                    BufferMessage::ChangeMode(from, to) => {
                        if !from.is_command() && to.is_command() {
                            update::update(to, &model.search, buffer, &BufferMessage::ResetCursor);

                            let prefix = match to {
                                Mode::Command(cmd) => match cmd {
                                    CommandMode::Command => Some(":".to_string()),
                                    CommandMode::Search(SearchDirection::Up) => {
                                        Some("?".to_string())
                                    }
                                    CommandMode::Search(SearchDirection::Down) => {
                                        Some("/".to_string())
                                    }
                                },
                                _ => None,
                            };

                            let bufferline = BufferLine {
                                prefix,
                                ..Default::default()
                            };

                            update::update(
                                to,
                                &model.search,
                                buffer,
                                &BufferMessage::SetContent(vec![bufferline]),
                            );
                        } else if from.is_command() && !to.is_command() {
                            update::update(
                                &model.mode,
                                &model.search,
                                buffer,
                                &BufferMessage::SetContent(vec![]),
                            );
                        }
                    }
                    &BufferMessage::Modification(
                        _,
                        TextModification::DeleteMotion(_, CursorDirection::Left),
                    ) => {
                        if let Some(line) = buffer.lines.last() {
                            if line.content.is_empty() {
                                actions.pop();
                                actions.push(Action::EmitMessages(vec![Message::Buffer(
                                    BufferMessage::ChangeMode(
                                        model.mode.clone(),
                                        get_mode_after_command(&model.mode_before),
                                    ),
                                )]));
                            }
                        }
                    }
                    _ => {}
                };

                update::update(&model.mode, &model.search, buffer, message);
            }

            actions
        }
        CommandLineState::WaitingForInput => {
            if message.is_some() {
                commandline.state = CommandLineState::Default;
            }

            let mut messages = Vec::new();
            if let Some(&BufferMessage::Modification(_, TextModification::Insert(cnt))) =
                message.as_ref()
            {
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
                    update::update(
                        &model.mode,
                        &model.search,
                        buffer,
                        &BufferMessage::SetContent(vec![]),
                    );

                    Message::Buffer(BufferMessage::ChangeMode(
                        model.mode.clone(),
                        get_mode_after_command(&model.mode_before),
                    ))
                };

                messages.push(Action::SkipRender);
                messages.push(Action::EmitMessages(vec![action]));
            }

            messages
        }
    }
}

pub fn update_on_execute(model: &mut Model) -> Vec<Action> {
    let mut actions = vec![Action::SkipRender];

    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;
    match commandline.state {
        CommandLineState::Default => {
            let is_search = matches!(
                model.mode,
                Mode::Command(CommandMode::Search(SearchDirection::Up))
                    | Mode::Command(CommandMode::Search(SearchDirection::Down))
            );

            let messages = if is_search {
                vec![
                    Message::Buffer(BufferMessage::ChangeMode(
                        model.mode.clone(),
                        get_mode_after_command(&model.mode_before),
                    )),
                    Message::Buffer(BufferMessage::MoveCursor(1, CursorDirection::Search(true))),
                ]
            } else if let Some(cmd) = buffer.lines.last() {
                vec![Message::ExecuteCommandString(cmd.content.clone())]
            } else {
                vec![Message::Buffer(BufferMessage::ChangeMode(
                    model.mode.clone(),
                    get_mode_after_command(&model.mode_before),
                ))]
            };

            update::update(
                &model.mode,
                &model.search,
                buffer,
                &BufferMessage::SetContent(vec![]),
            );

            actions.push(Action::EmitMessages(messages));
        }
        CommandLineState::WaitingForInput => {
            commandline.state = CommandLineState::Default;
            update::update(
                &model.mode,
                &model.search,
                buffer,
                &BufferMessage::SetContent(vec![]),
            );

            actions.push(Action::EmitMessages(vec![Message::Buffer(
                BufferMessage::ChangeMode(
                    model.mode.clone(),
                    get_mode_after_command(&model.mode_before),
                ),
            )]));
        }
    }

    actions
}

pub fn print(model: &mut Model, content: &[PrintContent]) -> Vec<Action> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    super::set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

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
        commandline.state = CommandLineState::WaitingForInput;

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

        vec![Action::EmitMessages(vec![Message::Buffer(
            BufferMessage::ChangeMode(model.mode.clone(), Mode::Command(CommandMode::Command)),
        )])]
    } else {
        Vec::new()
    };

    update::update(
        &model.mode,
        &model.search,
        &mut commandline.buffer,
        &BufferMessage::MoveCursor(1, CursorDirection::Bottom),
    );
    update::update(
        &model.mode,
        &model.search,
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
