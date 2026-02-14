use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Mode},
    view as buffer_view,
};

use crate::model::{FileTreeBuffer, FileTreeBufferSectionBuffer};

pub fn view(
    mode: &Mode,
    viewport: &ViewPort,
    buffer: &FileTreeBuffer,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
        .split(Rect {
            x: horizontal_offset,
            y: vertical_offset,
            width: viewport.width,
            height: viewport.height,
        });

    render_buffer(
        mode,
        frame,
        &buffer.parent,
        layout[0].x,
        layout[0].width,
        layout[0].y,
        layout[0].height,
    );

    let mut viewport = viewport.clone();
    viewport.height = layout[1].height;
    viewport.width = layout[1].width;

    buffer_view(
        &viewport,
        mode,
        &buffer.current.buffer,
        frame,
        layout[1].x,
        layout[1].y,
    );

    render_buffer(
        mode,
        frame,
        &buffer.preview,
        layout[2].x,
        layout[2].width,
        layout[2].y,
        layout[2].height,
    );
}

fn render_buffer(
    mode: &Mode,
    frame: &mut Frame,
    buffer_type: &FileTreeBufferSectionBuffer,
    horizontal_offset: u16,
    width: u16,
    vertical_offset: u16,
    height: u16,
) {
    let mut viewport = ViewPort {
        height,
        width,
        ..Default::default()
    };

    match buffer_type {
        FileTreeBufferSectionBuffer::Text(_, buffer) => {
            yeet_buffer::update_viewport_by_cursor(&mut viewport, buffer);

            buffer_view(
                &viewport,
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
            buffer_view(
                &viewport,
                mode,
                &Default::default(),
                frame,
                horizontal_offset,
                vertical_offset,
            );
        }
    };
}
