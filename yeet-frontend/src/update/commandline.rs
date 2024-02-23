use ratatui::style::Color;
use yeet_keymap::message::{Buffer, CursorDirection, Message, Mode, PrintContent};

use crate::{
    action::{Action, PostView, PreView},
    model::{
        buffer::{BufferLine, StylePartial},
        CommandLineState, Model,
    },
    task::Task,
};

use super::buffer::{self};

pub fn update(model: &mut Model, message: Option<&Buffer>) -> Vec<Action> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    super::set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

    match commandline.state {
        CommandLineState::Default => {
            if let Some(message) = message {
                if let Buffer::ChangeMode(from, to) = message {
                    if from != &Mode::Command && to == &Mode::Command {
                        buffer::reset_view(buffer);

                        let bufferline = BufferLine {
                            prefix: Some(":".to_string()),
                            ..Default::default()
                        };

                        buffer::set_content(to, buffer, vec![bufferline]);
                    } else if from == &Mode::Command && to != &Mode::Command {
                        buffer::set_content(&model.mode, buffer, vec![]);
                    }
                }

                buffer::update(&model.mode, buffer, message);
            }

            vec![
                Action::PreView(PreView::SkipRender),
                Action::PostView(PostView::Task(Task::EmitMessages(vec![Message::Rerender]))),
            ]
        }
        CommandLineState::WaitingForInput => {
            commandline.state = CommandLineState::Default;
            buffer::set_content(&model.mode, buffer, vec![]);

            let mut messages = vec![Message::Buffer(Buffer::ChangeMode(
                model.mode.clone(),
                Mode::default(),
            ))];

            // FIX: this emits Insert currently
            if let Some(message) = message {
                messages.push(Message::Buffer(message.clone()));
            }

            vec![
                Action::PreView(PreView::SkipRender),
                Action::PostView(PostView::Task(Task::EmitMessages(messages))),
            ]
        }
    }
}

pub fn update_on_execute(model: &mut Model) -> Option<Vec<Action>> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    match commandline.state {
        CommandLineState::Default => {
            let action = if let Some(cmd) = buffer.lines.last() {
                Message::ExecuteCommandString(cmd.content.clone())
            } else {
                Message::Buffer(Buffer::ChangeMode(model.mode.clone(), Mode::default()))
            };

            buffer::set_content(&model.mode, buffer, vec![]);

            Some(vec![
                Action::PreView(PreView::SkipRender),
                Action::PostView(PostView::Task(Task::EmitMessages(vec![action]))),
            ])
        }
        CommandLineState::WaitingForInput => {
            commandline.state = CommandLineState::Default;

            let bufferline = BufferLine {
                prefix: Some(":".to_string()),
                ..Default::default()
            };

            buffer.lines.pop();
            buffer.lines.push(bufferline);

            None
        }
    }
}

pub fn print(model: &mut Model, content: &[PrintContent]) -> Option<Vec<Action>> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    super::set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

    commandline.buffer.lines = content
        .iter()
        .map(|content| match content {
            PrintContent::Error(cntnt) => {
                let cntnt_len = cntnt.chars().count();
                BufferLine {
                    content: cntnt.to_string(),
                    style: vec![(0, cntnt_len, StylePartial::Foreground(Color::Red))],
                    ..Default::default()
                }
            }
            PrintContent::Info(cntnt) => BufferLine {
                content: cntnt.to_string(),
                ..Default::default()
            },
        })
        .collect();

    let actions = if commandline.buffer.lines.len() > 1 {
        commandline.state = CommandLineState::WaitingForInput;

        let content = "Press ENTER or type command to continue";
        commandline.buffer.lines.push(BufferLine {
            content: content.to_string(),
            style: vec![(
                0,
                content.chars().count(),
                StylePartial::Foreground(Color::LightBlue),
            )],
            ..Default::default()
        });

        Some(vec![Action::PostView(PostView::Task(Task::EmitMessages(
            vec![Message::Buffer(Buffer::ChangeMode(
                model.mode.clone(),
                Mode::Command,
            ))],
        )))])
    } else {
        None
    };

    buffer::update(
        &model.mode,
        &mut commandline.buffer,
        &Buffer::MoveCursor(1, CursorDirection::Bottom),
    );
    buffer::update(
        &model.mode,
        &mut commandline.buffer,
        &Buffer::MoveCursor(1, CursorDirection::LineEnd),
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
