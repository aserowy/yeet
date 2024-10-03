use ratatui::{layout::Rect, Frame};
use ratatui_image::Image;
use yeet_buffer::{model::Mode, view};

use crate::{
    error::AppError,
    model::{BufferType, Model},
    terminal::TerminalWrapper,
};

mod commandline;
mod statusline;

pub fn render_model(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    terminal.draw(|frame| {
        let layout = model.layout.clone();

        commandline::view(model, frame);

        view::view(
            &model.mode,
            &model.files.current.buffer,
            frame,
            layout.current,
        );

        render_buffer(&model.mode, frame, layout.parent, &model.files.parent);
        render_buffer(&model.mode, frame, layout.preview, &model.files.preview);

        statusline::view(model, frame, layout.statusline);
    })
}

fn render_buffer(mode: &Mode, frame: &mut Frame, layout: Rect, buffer_type: &BufferType) {
    match buffer_type {
        BufferType::Text(_, buffer) => {
            view::view(mode, &buffer, frame, layout);
        }
        BufferType::Image(_, protocol) => {
            frame.render_widget(Image::new(protocol.as_ref()), layout);
        }
        BufferType::None => {}
    };
}
