use ratatui::{
    prelude::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::model::Model;

pub fn view(model: &mut Model, frame: &mut Frame, rect: Rect) {
    let content = match model.current_path.to_str() {
        Some(str) => str,
        None => "",
    };

    let style = Style::default().fg(Color::Green);
    let span = Span::styled(content, style);

    frame.render_widget(Paragraph::new(Line::from(span)), rect);
}
