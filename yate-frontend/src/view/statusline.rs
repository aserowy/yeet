use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::model::{buffer::undo::BufferChanged, Model};

pub fn view(model: &mut Model, frame: &mut Frame, rect: Rect) {
    let changes = get_changes_content(model);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Max(changes.width() as u16)])
        .split(rect);

    let content = model.current_path.to_str().unwrap_or("");
    let style = Style::default().fg(Color::Green);
    let span = Span::styled(content, style);

    frame.render_widget(Paragraph::new(Line::from(span)), layout[0]);
    frame.render_widget(Paragraph::new(changes), layout[1]);
}

fn get_changes_content(model: &Model) -> Line {
    let modifications = model.current_directory.undo.get_uncommited_changes();
    let changes = crate::model::buffer::undo::consolidate(&modifications);

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
