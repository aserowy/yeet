use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Mode},
    view as buffer_view,
};

use crate::model::{DirectoryBuffer, PreviewImageBuffer};

pub enum PreviewView<'a> {
    Directory(&'a DirectoryBuffer),
    Image(&'a PreviewImageBuffer),
    None,
}

pub struct FileTreeView<'a> {
    pub mode: &'a Mode,
    pub parent_viewport: &'a ViewPort,
    pub current_viewport: &'a ViewPort,
    pub preview_viewport: &'a ViewPort,
    pub parent_buffer: &'a DirectoryBuffer,
    pub current_buffer: &'a DirectoryBuffer,
    pub preview_buffer: PreviewView<'a>,
    pub horizontal_offset: u16,
    pub vertical_offset: u16,
}

pub fn view(view: FileTreeView<'_>, frame: &mut Frame) {
    let FileTreeView {
        mode,
        parent_viewport,
        current_viewport,
        preview_viewport,
        parent_buffer,
        current_buffer,
        preview_buffer,
        horizontal_offset,
        vertical_offset,
    } = view;
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(Constraint::from_ratios([(1, 5), (2, 5), (2, 5)]))
        .split(Rect {
            x: horizontal_offset,
            y: vertical_offset,
            width: current_viewport.width,
            height: current_viewport.height,
        });

    render_directory_buffer(
        mode,
        frame,
        parent_viewport,
        parent_buffer,
        layout[0].x,
        layout[0].y,
    );

    buffer_view(
        current_viewport,
        mode,
        &current_buffer.buffer,
        frame,
        layout[1].x,
        layout[1].y,
    );

    match preview_buffer {
        PreviewView::Directory(buffer) => {
            render_directory_buffer(
                mode,
                frame,
                preview_viewport,
                buffer,
                layout[2].x,
                layout[2].y,
            );
        }
        PreviewView::Image(buffer) => {
            let rect = Rect {
                x: layout[2].x,
                y: layout[2].y,
                width: preview_viewport.width,
                height: preview_viewport.height,
            };

            frame.render_widget(Image::new(&buffer.protocol), rect);
        }
        PreviewView::None => {
            render_directory_buffer(
                mode,
                frame,
                preview_viewport,
                &DirectoryBuffer::default(),
                layout[2].x,
                layout[2].y,
            );
        }
    }
}

fn render_directory_buffer(
    mode: &Mode,
    frame: &mut Frame,
    viewport: &ViewPort,
    buffer: &DirectoryBuffer,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    let mut viewport = viewport.clone();
    yeet_buffer::update_viewport_by_cursor(&mut viewport, &buffer.buffer);

    buffer_view(
        &viewport,
        mode,
        &buffer.buffer,
        frame,
        horizontal_offset,
        vertical_offset,
    );
}
