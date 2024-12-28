use ratatui::layout::{Constraint, Direction, Layout, Rect};
use yeet_buffer::{
    message::{BufferMessage, ViewPortDirection},
    model::Mode,
};

use crate::{
    action::Action,
    error::AppError,
    model::{history::History, App, FileTreeBuffer, FileTreeBufferSection, Window},
};

use super::{history, selection};

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
    let (vp, _, _) = match window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(vp, cursor, id) => (vp, cursor, id),
    };

    vp.height = area.height;
    vp.width = area.width;

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

// NOTE: FileTreeBuffer related
pub fn relocate(
    history: &History,
    mode: &Mode,
    buffer: &mut FileTreeBuffer,
    direction: &ViewPortDirection,
) -> Vec<Action> {
    let msg = BufferMessage::MoveViewPort(direction.clone());

    yeet_buffer::update::update_buffer(
        &mut buffer.current_vp,
        &mut buffer.current_cursor,
        mode,
        &mut buffer.current.buffer,
        &msg,
    );

    let mut actions = Vec::new();
    if let Some(path) = selection::get_current_selected_path(buffer) {
        let selection = history::get_selection_from_history(history, &path).map(|s| s.to_owned());
        actions.push(Action::Load(
            FileTreeBufferSection::Preview,
            path,
            selection,
        ));
    }

    actions
}
