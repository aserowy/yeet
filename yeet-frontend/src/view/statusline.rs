use std::path::PathBuf;

use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use yeet_buffer::model::{
    undo::{self, BufferChanged},
    viewport::ViewPort,
};

use crate::model::{self, Buffer, DirectoryBuffer};

pub fn view(current: &Buffer, viewport: &ViewPort, frame: &mut Frame, rect: Rect) {
    match current {
        Buffer::Directory(it) => filetree_status(it, viewport, frame, rect),
        Buffer::Image(_) | Buffer::Content(_) | Buffer::PathReference(_) | Buffer::Empty => {}
    }
}

fn filetree_status(buffer: &DirectoryBuffer, viewport: &ViewPort, frame: &mut Frame, rect: Rect) {
    let selected = model::get_selected_path(buffer, &viewport.cursor);
    let permissions = get_permissions(&selected);

    let changes = get_changes_content(buffer);
    let position = get_position_content(buffer, viewport);

    let content = buffer.path.to_str().unwrap_or("");
    let style = Style::default().fg(Color::Gray);
    let span = Span::styled(content, style);
    let path = Line::from(span);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(path.width() as u16),
            Constraint::Length(3),
            Constraint::Length(permissions.width() as u16),
            Constraint::Length(3),
            Constraint::Length(changes.width() as u16),
            Constraint::Min(3),
            Constraint::Length(position.width() as u16),
        ])
        .split(rect);

    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        rect,
    );

    frame.render_widget(Paragraph::new(path), layout[0]);
    frame.render_widget(Paragraph::new(permissions), layout[2]);
    frame.render_widget(Paragraph::new(changes), layout[4]);
    frame.render_widget(Paragraph::new(position), layout[6]);
}

fn get_position_content<'a>(buffer: &'a DirectoryBuffer, viewport: &ViewPort) -> Line<'a> {
    let count = buffer.buffer.lines.len();
    let mut position = viewport.cursor.vertical_index + 1;

    let mut content = Vec::new();
    if count == 0 {
        position = 0;
    }

    content.push(Span::styled(
        format!("{}/", position),
        Style::default().fg(Color::Gray),
    ));

    content.push(Span::styled(
        format!("{}", count),
        Style::default().fg(Color::Gray),
    ));

    Line::from(content)
}

fn get_changes_content(buffer: &DirectoryBuffer) -> Line<'_> {
    let modifications = buffer.buffer.undo.get_uncommited_changes();
    let changes = undo::consolidate_modifications(&modifications);

    let (mut added, mut changed, mut removed) = (0, 0, 0);
    for change in changes {
        match change {
            BufferChanged::Content(_, _, _) => changed += 1,
            BufferChanged::LineAdded(_, _) => added += 1,
            BufferChanged::LineRemoved(_, _) => removed += 1,
        }
    }

    let mut content = Vec::new();
    if added > 0 {
        content.push(Span::styled(
            format!("+{} ", added),
            Style::default().fg(Color::Green),
        ));
    }

    if changed > 0 {
        content.push(Span::styled(
            format!("~{} ", changed),
            Style::default().fg(Color::Yellow),
        ));
    }

    if removed > 0 {
        content.push(Span::styled(
            format!("-{} ", removed),
            Style::default().fg(Color::Red),
        ));
    }

    Line::from(content)
}

#[cfg(target_os = "windows")]
fn get_permissions(path: Option<PathBuf>) -> Line<'_> {
    Ok(Line::from("".to_string()))
}

#[cfg(not(target_os = "windows"))]
fn get_permissions(path: &Option<PathBuf>) -> Line<'_> {
    use std::{fs::File, os::unix::fs::PermissionsExt};

    let empty = Line::from("---------".to_string());
    let path = match path {
        Some(it) => it,
        None => return empty,
    };

    let file = match File::open(path) {
        Ok(it) => it,
        Err(_) => return empty,
    };

    let metadata = match file.metadata() {
        Ok(it) => it,
        Err(_) => return empty,
    };

    let permissions = metadata.permissions().mode();
    let mut result = String::new();
    for i in (0..9).rev() {
        let bit = (permissions >> i) & 1;
        let char = match i % 3 {
            2 => {
                if bit == 1 {
                    'r'
                } else {
                    '-'
                }
            }
            1 => {
                if bit == 1 {
                    'w'
                } else {
                    '-'
                }
            }
            0 => {
                if bit == 1 {
                    'x'
                } else {
                    '-'
                }
            }
            _ => unreachable!(),
        };
        result.push(char);
    }
    Line::from(result)
}
