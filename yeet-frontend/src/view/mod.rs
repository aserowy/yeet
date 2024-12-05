use ratatui::{layout::Rect, Frame};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Cursor, Mode},
    view,
};

use crate::{
    error::AppError,
    model::{Buffer, FileTreeBufferSectionBuffer, Model},
    terminal::TerminalWrapper,
};

mod commandline;
mod statusline;

pub fn render_model(terminal: &mut TerminalWrapper, model: &Model) -> Result<(), AppError> {
    let buffer = match &model.buffer {
        Buffer::FileTree(it) => it,
        Buffer::Text(_) => todo!(),
    };

    terminal.draw(|frame| {
        let layout = model.layout.clone();

        commandline::view(model, frame);

        view::view(
            &buffer.current_vp,
            &buffer.current_cursor,
            &model.mode,
            &buffer.current.buffer,
            &buffer.show_border,
            frame,
            layout.current,
        );

        render_buffer(
            &buffer.parent_vp,
            &buffer.parent_cursor,
            &model.mode,
            frame,
            layout.parent,
            &buffer.parent,
            &buffer.show_border,
        );
        render_buffer(
            &buffer.preview_vp,
            &buffer.preview_cursor,
            &model.mode,
            frame,
            layout.preview,
            &buffer.preview,
            &false,
        );

        statusline::view(buffer, frame, layout.statusline);
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
