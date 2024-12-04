use ratatui::{layout::Rect, Frame};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Cursor, Mode},
    view,
};

use crate::{
    error::AppError,
    model::{FileTreeBufferSectionBuffer, Model},
    terminal::TerminalWrapper,
};

mod commandline;
mod statusline;

pub fn render_model(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    terminal.draw(|frame| {
        let layout = model.layout.clone();

        commandline::view(model, frame);

        view::view(
            &model.files.current_vp,
            &model.files.current_cursor,
            &model.mode,
            &model.files.current.buffer,
            &model.files.show_border,
            frame,
            layout.current,
        );

        render_buffer(
            &model.files.parent_vp,
            &model.files.parent_cursor,
            &model.mode,
            frame,
            layout.parent,
            &model.files.parent,
            &model.files.show_border,
        );
        render_buffer(
            &model.files.preview_vp,
            &model.files.preview_cursor,
            &model.mode,
            frame,
            layout.preview,
            &model.files.preview,
            &false,
        );

        statusline::view(model, frame, layout.statusline);
    })
}

fn render_buffer(
    viewport: &ViewPort,
    cursor: &Option<Cursor>,
    mode: &Mode,
    frame: &mut Frame,
    layout: Rect,
    buffer_type: &FileTreeBufferSectionBuffer,
    show_border: &bool,
) {
    match buffer_type {
        FileTreeBufferSectionBuffer::Text(_, buffer) => {
            view::view(viewport, cursor, mode, buffer, show_border, frame, layout);
        }
        FileTreeBufferSectionBuffer::Image(_, protocol) => {
            frame.render_widget(Image::new(protocol), layout);
        }
        FileTreeBufferSectionBuffer::None => {
            view::view(
                viewport,
                &None,
                mode,
                &Default::default(),
                show_border,
                frame,
                layout,
            );
        }
    };
}
