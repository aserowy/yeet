use ratatui::{layout::Rect, widgets::Paragraph, Frame};
use yeet_buffer::{model::Mode, view_themed as buffer_view};

use crate::{error::AppError, model::CommandLine, theme::Theme};

pub fn view(commandline: &CommandLine, mode: &Mode, theme: &Theme, frame: &mut Frame) -> Result<(), AppError> {
    let buffer_theme = theme.to_buffer_theme();
    buffer_view(&commandline.viewport, mode, &commandline.buffer, &buffer_theme, frame);

    let rect = Rect {
        x: commandline.viewport.width,
        y: commandline.viewport.y,
        width: u16::try_from(commandline.key_sequence.chars().count())?,
        height: commandline.viewport.height,
    };

    frame.render_widget(Paragraph::new(commandline.key_sequence.clone()), rect);

    Ok(())
}
