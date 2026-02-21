use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use yeet_buffer::model::undo::{self, BufferChanged};

use crate::model::{Buffer, DirectoryBuffer};

pub fn view(current: &Buffer, frame: &mut Frame, rect: Rect) {
    match current {
        Buffer::Directory(it) => filetree_status(it, frame, rect),
        Buffer::Image(_) | Buffer::Content(_) | Buffer::Empty => {}
    }
}

fn filetree_status(buffer: &DirectoryBuffer, frame: &mut Frame, rect: Rect) {
    let changes = get_changes_content(buffer);
    let position = get_position_content(buffer);

    let content = buffer.path.to_str().unwrap_or("");
    let style = Style::default().fg(Color::Gray);
    let span = Span::styled(content, style);
    let path = Line::from(span);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(path.width() as u16),
            Constraint::Length(3),
            Constraint::Min(changes.width() as u16),
            Constraint::Length(position.width() as u16),
        ])
        .split(rect);

    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Black)),
        rect,
    );

    frame.render_widget(Paragraph::new(path), layout[0]);
    frame.render_widget(Paragraph::new(changes), layout[2]);
    frame.render_widget(Paragraph::new(position), layout[3]);
}

fn get_position_content<'a>(buffer: &'a DirectoryBuffer) -> Line<'a> {
    let count = buffer.buffer.lines.len();
    let mut position = buffer.buffer.cursor.vertical_index + 1;

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
