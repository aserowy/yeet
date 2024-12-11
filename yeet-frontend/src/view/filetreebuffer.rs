use ratatui::{layout::Rect, Frame};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Cursor, Mode},
    view,
};

use crate::model::{FileTreeBuffer, FileTreeBufferSectionBuffer};

pub fn view(
    mode: &Mode,
    buffer: &FileTreeBuffer,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    render_buffer(
        &buffer.parent_vp,
        &buffer.parent_cursor,
        mode,
        frame,
        &buffer.parent,
        horizontal_offset,
        vertical_offset,
    );

    view::view(
        &buffer.current_vp,
        &buffer.current_cursor,
        mode,
        &buffer.current.buffer,
        frame,
        horizontal_offset + &buffer.parent_vp.width,
        vertical_offset,
    );

    render_buffer(
        &buffer.preview_vp,
        &buffer.preview_cursor,
        mode,
        frame,
        &buffer.preview,
        horizontal_offset + &buffer.parent_vp.width + &buffer.current_vp.width,
        vertical_offset,
    );
}

fn render_buffer(
    viewport: &ViewPort,
    cursor: &Option<Cursor>,
    mode: &Mode,
    frame: &mut Frame,
    buffer_type: &FileTreeBufferSectionBuffer,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    match buffer_type {
        FileTreeBufferSectionBuffer::Text(_, buffer) => {
            view::view(
                viewport,
                cursor,
                mode,
                buffer,
                frame,
                horizontal_offset,
                vertical_offset,
            );
        }
        FileTreeBufferSectionBuffer::Image(_, protocol) => {
            let rect = Rect {
                x: horizontal_offset,
                y: vertical_offset,
                width: viewport.width,
                height: viewport.height,
            };

            frame.render_widget(Image::new(protocol), rect);
        }
        FileTreeBufferSectionBuffer::None => {
            view::view(
                viewport,
                &None,
                mode,
                &Default::default(),
                frame,
                horizontal_offset,
                vertical_offset,
            );
        }
    };
}
