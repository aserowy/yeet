use ratatui::style::Color;
use yeet_keymap::message::{Buffer, CursorDirection, Message, Mode, PrintContent};

use crate::{
    action::{Action, PostView},
    layout::CommandLineLayout,
    model::{
        buffer::{BufferLine, StylePartial},
        CommandLineState, Model,
    },
    task::Task,
};

use super::buffer::{self, cursor};

pub fn update(model: &mut Model, message: Option<&Buffer>) -> Option<Vec<Action>> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    let sequence_len = model.key_sequence.chars().count() as u16;
    commandline.layout = CommandLineLayout::new(model.layout.commandline, sequence_len);

    super::set_viewport_dimensions(&mut buffer.view_port, &commandline.layout.buffer);

    if let Some(message) = message {
        if let Buffer::ChangeMode(from, to) = message {
            if to != &Mode::Command {
                buffer.lines = vec![BufferLine {
                    content: format!("--{}--", model.mode.to_string().to_uppercase()),
                    ..Default::default()
                }];

                cursor::validate(to, buffer);
            }

            if from != &Mode::Command && to == &Mode::Command {
                buffer::reset_view(buffer);

                let bufferline = BufferLine {
                    prefix: Some(":".to_string()),
                    ..Default::default()
                };

                buffer::set_content(to, buffer, vec![bufferline]);
            }
        }

        buffer::update(&model.mode, buffer, message);
    }

    None
}

pub fn print(model: &mut Model, content: &Vec<PrintContent>) -> Option<Vec<Action>> {
    let commandline = &mut model.commandline;
    let buffer = &mut commandline.buffer;

    commandline.layout = CommandLineLayout::new(model.layout.commandline, 0);

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
