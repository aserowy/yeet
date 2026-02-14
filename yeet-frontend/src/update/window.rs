use crate::{
    error::AppError,
    model::{App, Window},
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn update(app: &mut App, area: Rect) -> Result<(), AppError> {
    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Length(1),
            Constraint::Length(u16::try_from(app.commandline.buffer.lines.len())?),
        ])
        .split(area);

    set_buffer_vp(&mut app.window, main[0])?;
    set_commandline_vp(&mut app.commandline, main[2])?;

    Ok(())
}

fn set_buffer_vp(window: &mut Window, area: Rect) -> Result<(), AppError> {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
        .split(area);

    let parent_rect = layout[0];
    let current_rect = layout[1];
    let preview_rect = layout[2];

    let (parent_vp, current_vp, preview_vp) = match window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(parent, current, preview) => (parent, current, preview),
    };

    parent_vp.height = parent_rect.height;
    parent_vp.width = parent_rect.width;
    parent_vp.x = parent_rect.x;
    parent_vp.y = parent_rect.y;

    current_vp.height = current_rect.height;
    current_vp.width = current_rect.width;
    current_vp.x = current_rect.x;
    current_vp.y = current_rect.y;

    preview_vp.height = preview_rect.height;
    preview_vp.width = preview_rect.width;
    preview_vp.x = preview_rect.x;
    preview_vp.y = preview_rect.y;

    Ok(())
}

fn set_commandline_vp(
    commandline: &mut crate::model::CommandLine,
    rect: Rect,
) -> Result<(), AppError> {
    commandline.viewport.height = rect.height;

    let key_sequence_offset = u16::try_from(commandline.key_sequence.chars().count())?;
    commandline.viewport.width = rect.width - key_sequence_offset;

    Ok(())
}
