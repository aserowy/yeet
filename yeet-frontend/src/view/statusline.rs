use std::path::PathBuf;

use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use yeet_buffer::model::{undo, undo::BufferChanged, viewport::ViewPort};

use crate::{
    model::{self, Buffer, DirectoryBuffer, TasksBuffer},
    theme::{tokens, Theme},
};

pub fn view(
    current: &Buffer,
    viewport: &ViewPort,
    frame: &mut Frame,
    rect: Rect,
    is_focused: bool,
    theme: &Theme,
) {
    let rect = if viewport.show_border {
        let block = Block::default().borders(Borders::RIGHT).border_style(
            theme.style_fg_bg(tokens::STATUSLINE_BORDER_FG, tokens::STATUSLINE_BORDER_BG),
        );

        let inner = block.inner(rect);

        frame.render_widget(block, rect);

        inner
    } else {
        rect
    };

    match current {
        Buffer::Directory(it) => {
            if is_focused {
                filetree_status(it, viewport, frame, rect, theme)
            } else {
                filetree_status_unfocused(it, frame, rect, theme)
            }
        }
        Buffer::Tasks(it) => {
            if is_focused {
                tasks_status(it, viewport, frame, rect, theme)
            } else {
                tasks_status_unfocused(frame, rect, theme)
            }
        }
        Buffer::Image(_) | Buffer::Content(_) | Buffer::PathReference(_) | Buffer::Empty => {}
    }
}

fn tasks_status(
    buffer: &TasksBuffer,
    viewport: &ViewPort,
    frame: &mut Frame,
    rect: Rect,
    theme: &Theme,
) {
    let count = buffer.buffer.lines.len();
    let position = if count == 0 {
        0
    } else {
        viewport.cursor.vertical_index + 1
    };

    let label = Line::from(Span::styled(
        "Tasks",
        theme
            .style_fg(tokens::STATUSLINE_FOCUSED_FG)
            .add_modifier(Modifier::BOLD),
    ));
    let position_line = Line::from(vec![
        Span::styled(
            format!("{}/", position),
            theme.style_fg(tokens::STATUSLINE_POSITION_FG),
        ),
        Span::styled(
            format!("{}", count),
            theme.style_fg(tokens::STATUSLINE_POSITION_FG),
        ),
    ]);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(label.width() as u16),
            Constraint::Min(3),
            Constraint::Length(position_line.width() as u16),
        ])
        .split(rect);

    frame.render_widget(
        Block::default().style(theme.style_bg(tokens::STATUSLINE_BG)),
        rect,
    );

    frame.render_widget(Paragraph::new(label), layout[0]);
    frame.render_widget(Paragraph::new(position_line), layout[2]);
}

fn tasks_status_unfocused(frame: &mut Frame, rect: Rect, theme: &Theme) {
    let label = Line::from(Span::styled(
        "Tasks",
        theme.style_fg(tokens::STATUSLINE_UNFOCUSED_FG),
    ));

    frame.render_widget(
        Block::default().style(theme.style_bg(tokens::STATUSLINE_BG)),
        rect,
    );
    frame.render_widget(Paragraph::new(label), rect);
}

fn filetree_status(
    buffer: &DirectoryBuffer,
    viewport: &ViewPort,
    frame: &mut Frame,
    rect: Rect,
    theme: &Theme,
) {
    let selected = model::get_selected_path(buffer, &viewport.cursor);
    let permissions =
        get_permissions(&selected).patch_style(theme.style_fg(tokens::STATUSLINE_PERMISSIONS_FG));

    let changes = get_changes_content(buffer, theme);
    let position = get_position_content(buffer, viewport, theme);

    let path = Line::from(Span::styled(
        buffer.path.to_str().unwrap_or(""),
        theme
            .style_fg(tokens::STATUSLINE_FOCUSED_FG)
            .add_modifier(Modifier::BOLD),
    ));

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
        Block::default().style(theme.style_bg(tokens::STATUSLINE_BG)),
        rect,
    );

    frame.render_widget(Paragraph::new(path), layout[0]);
    frame.render_widget(Paragraph::new(permissions), layout[2]);
    frame.render_widget(Paragraph::new(changes), layout[4]);
    frame.render_widget(Paragraph::new(position), layout[6]);
}

fn filetree_status_unfocused(
    buffer: &DirectoryBuffer,
    frame: &mut Frame,
    rect: Rect,
    theme: &Theme,
) {
    let content = buffer.path.to_str().unwrap_or("");
    let style = theme.style_fg(tokens::STATUSLINE_UNFOCUSED_FG);
    let path = Line::from(Span::styled(content, style));

    frame.render_widget(
        Block::default().style(theme.style_bg(tokens::STATUSLINE_BG)),
        rect,
    );
    frame.render_widget(Paragraph::new(path), rect);
}

fn get_position_content<'a>(
    buffer: &'a DirectoryBuffer,
    viewport: &ViewPort,
    theme: &Theme,
) -> Line<'a> {
    let count = buffer.buffer.lines.len();
    let mut position = viewport.cursor.vertical_index + 1;

    let mut content = Vec::new();
    if count == 0 {
        position = 0;
    }

    content.push(Span::styled(
        format!("{}/", position),
        theme.style_fg(tokens::STATUSLINE_POSITION_FG),
    ));

    content.push(Span::styled(
        format!("{}", count),
        theme.style_fg(tokens::STATUSLINE_POSITION_FG),
    ));

    Line::from(content)
}

fn get_changes_content<'a>(buffer: &'a DirectoryBuffer, theme: &Theme) -> Line<'a> {
    let modifications = buffer.buffer.uncommitted_changes();
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
            theme.style_fg(tokens::DIFF_ADDED),
        ));
    }

    if changed > 0 {
        content.push(Span::styled(
            format!("~{} ", changed),
            theme.style_fg(tokens::DIFF_MODIFIED),
        ));
    }

    if removed > 0 {
        content.push(Span::styled(
            format!("-{} ", removed),
            theme.style_fg(tokens::DIFF_REMOVED),
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
