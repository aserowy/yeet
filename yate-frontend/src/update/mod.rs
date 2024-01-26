use std::path::{Path, PathBuf};

use ratatui::{prelude::Rect, style::Color};
use yate_keymap::message::{Message, Mode, ViewPortDirection};

use crate::{
    layout::AppLayout,
    model::{
        buffer::{Buffer, BufferLine, Cursor, CursorPosition, StylePartial},
        Model,
    },
};

mod buffer;

// TODO: refactor file!!!!1111eleven
pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    match message {
        Message::ChangeKeySequence(sequence) => {
            model.key_sequence = sequence.clone();
        }
        Message::ChangeMode(from, to) => {
            model.mode = to.clone();

            match from {
                Mode::Command => {
                    unfocus_buffer(&mut model.commandline);
                    update_commandline(model, layout, message);
                }
                Mode::Normal => {
                    unfocus_buffer(&mut model.current_directory);
                }
            }

            match to {
                Mode::Command => {
                    focus_buffer(&mut model.commandline);
                    update_commandline(model, layout, message);
                }
                Mode::Normal => {
                    focus_buffer(&mut model.current_directory);
                }
            }
        }
        Message::ExecuteCommand => todo!(),
        Message::Modification(_) => match model.mode {
            Mode::Normal => {
                // NOTE: add file modification handling
                update_current_directory(model, layout, message);
            }
            Mode::Command => {
                update_commandline(model, layout, message);
            }
        },
        Message::MoveCursor(_, _) => match model.mode {
            Mode::Normal => {
                update_current_directory(model, layout, message);
                update_preview(model, layout, message);
            }
            Mode::Command => {
                update_commandline(model, layout, message);
            }
        },
        Message::MoveViewPort(_) => match model.mode {
            Mode::Normal => {
                update_current_directory(model, layout, message);
                update_preview(model, layout, message);
            }
            Mode::Command => {
                update_commandline(model, layout, message);
            }
        },
        Message::Refresh => {
            update_commandline(model, layout, message);
            update_current_directory(model, layout, message);
            update_parent_directory(model, layout, message);
            update_preview(model, layout, message);
        }
        Message::SelectCurrent => {
            if let Some(target) = get_target_path(model) {
                if !target.is_dir() {
                    return;
                }

                model.current_path = target;

                update_current_directory(model, layout, message);
                update_parent_directory(model, layout, message);
                update_preview(model, layout, message);
            }
        }
        Message::SelectParent => {
            if let Some(parent) = &model.current_path.parent() {
                model.current_path = parent.to_path_buf();

                update_current_directory(model, layout, message);
                update_parent_directory(model, layout, message);
                update_preview(model, layout, message);
            }
        }
        Message::Quit => {}
    }
}

fn update_commandline(model: &mut Model, layout: &AppLayout, message: &Message) {
    let buffer = &mut model.commandline;
    let layout = &layout.commandline;

    buffer.view_port.height = usize::from(layout.height);
    buffer.view_port.width = usize::from(layout.width);

    if let Message::ChangeMode(from, to) = message {
        if from == &Mode::Command && to != &Mode::Command {
            buffer.lines = vec![BufferLine::default()];
        }

        if from != &Mode::Command && to == &Mode::Command {
            buffer::reset_view(&mut buffer.view_port, &mut buffer.cursor);

            let bufferline = BufferLine {
                prefix: Some(":".to_string()),
                ..Default::default()
            };

            buffer.lines = vec![bufferline];
        }
    }

    buffer::update(buffer, message);
}

fn focus_buffer(buffer: &mut Buffer) {
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor = false;
    }
}

fn unfocus_buffer(buffer: &mut Buffer) {
    if let Some(cursor) = &mut buffer.cursor {
        cursor.hide_cursor = true;
    }
}

fn update_current_directory(model: &mut Model, layout: &AppLayout, message: &Message) {
    update_buffer_with_path(
        &mut model.current_directory,
        &layout.current_directory,
        message,
        &model.current_path,
    );
}

fn update_parent_directory(model: &mut Model, layout: &AppLayout, message: &Message) {
    let path = Path::new(&model.current_path);
    match path.parent() {
        Some(parent) => {
            update_buffer_with_path(
                &mut model.parent_directory,
                &layout.parent_directory,
                message,
                parent,
            );

            let current_filename = path.file_name().unwrap().to_str().unwrap();
            if let Some(index) = model
                .parent_directory
                .lines
                .iter()
                .position(|line| line.content == current_filename)
            {
                if let Some(cursor) = &mut model.parent_directory.cursor {
                    cursor.vertical_index = index;
                } else {
                    model.parent_directory.cursor = Some(Cursor {
                        horizontial_index: CursorPosition::None,
                        vertical_index: index,
                        ..Default::default()
                    });
                }
            }

            buffer::update(
                &mut model.parent_directory,
                &Message::MoveViewPort(ViewPortDirection::CenterOnCursor),
            );
        }
        None => model.parent_directory.lines = vec![],
    }
}

fn update_preview(model: &mut Model, layout: &AppLayout, message: &Message) {
    if let Some(target) = get_target_path(model) {
        update_buffer_with_path(&mut model.preview, &layout.preview, message, &target);
    }
}

fn update_buffer_with_path(buffer: &mut Buffer, layout: &Rect, message: &Message, path: &Path) {
    buffer.view_port.height = usize::from(layout.height);
    buffer.view_port.width = usize::from(layout.width);

    let content = if path.is_dir() {
        get_directory_content(path)
    } else {
        Vec::new()
    };

    buffer.lines = content;

    buffer::update(buffer, message);
}

fn get_target_path(model: &Model) -> Option<PathBuf> {
    let buffer = &model.current_directory;
    if buffer.lines.is_empty() {
        return None;
    }

    if let Some(cursor) = &buffer.cursor {
        let current = &buffer.lines[cursor.vertical_index];
        let target = model.current_path.join(&current.content);

        if target.exists() {
            Some(target)
        } else {
            None
        }
    } else {
        None
    }
}

fn get_directory_content(path: &Path) -> Vec<BufferLine> {
    let mut content: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|entry| get_bufferline_by_path(&entry.unwrap().path()))
        .collect();

    content.sort_unstable_by(|a, b| {
        a.content
            .to_ascii_uppercase()
            .cmp(&b.content.to_ascii_uppercase())
    });

    content
}

fn get_bufferline_by_path(path: &Path) -> BufferLine {
    let content = path.file_name().unwrap().to_str().unwrap().to_string();

    let style = if path.is_dir() {
        vec![(
            0,
            content.chars().count(),
            StylePartial::Foreground(Color::LightBlue),
        )]
    } else {
        vec![]
    };

    BufferLine {
        content,
        style,
        ..Default::default()
    }
}
