use ratatui_image::Image;
use yeet_buffer::view;

use crate::{
    error::AppError,
    model::{Model, PreviewContent},
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
        view::view(
            &model.mode,
            &model.files.parent.buffer,
            frame,
            layout.parent,
        );

        match &model.files.preview {
            PreviewContent::Buffer(dir) => {
                view::view(&model.mode, &dir.buffer, frame, layout.preview);
            }
            PreviewContent::Image(_, protocol) => {
                frame.render_widget(Image::new(protocol.as_ref()), layout.preview);
            }
            PreviewContent::None => {}
        };

        statusline::view(model, frame, layout.statusline);
    })
}
