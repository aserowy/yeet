use ratatui::{layout::Rect, Frame};
use ratatui_image::Image;
use yeet_buffer::{
    model::{viewport::ViewPort, Mode},
    view as buffer_view,
};

use crate::model::{App, Buffer, DirectoryBuffer, PreviewImageBuffer, Window};

pub enum PreviewView<'a> {
    Directory(&'a DirectoryBuffer),
    Image(&'a PreviewImageBuffer),
    None,
}

pub fn view(
    mode: &Mode,
    app: &App,
    frame: &mut Frame,
    horizontal_offset: u16,
    vertical_offset: u16,
) {
    let (parent_viewport, current_viewport, preview_viewport) = match &app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(parent, current, preview) => (parent, current, preview),
    };

    let parent_buffer =
        app.buffers
            .get(&parent_viewport.buffer_id)
            .and_then(|buffer| match buffer {
                Buffer::Directory(it) => Some(it),
                _ => None,
            });
    let current_buffer =
        app.buffers
            .get(&current_viewport.buffer_id)
            .and_then(|buffer| match buffer {
                Buffer::Directory(it) => Some(it),
                _ => None,
            });
    let preview_buffer = app
        .buffers
        .get(&preview_viewport.buffer_id)
        .map(|buffer| match buffer {
            Buffer::Directory(it) => PreviewView::Directory(it),
            Buffer::PreviewImage(it) => PreviewView::Image(it),
            Buffer::_Text(_) => PreviewView::None,
        })
        .unwrap_or(PreviewView::None);

    let (parent_buffer, current_buffer) = match (parent_buffer, current_buffer) {
        (Some(parent), Some(current)) => (parent, current),
        _ => return,
    };

    let parent_x = parent_viewport.x.saturating_add(horizontal_offset);
    let parent_y = parent_viewport.y.saturating_add(vertical_offset);
    let current_x = current_viewport.x.saturating_add(horizontal_offset);
    let current_y = current_viewport.y.saturating_add(vertical_offset);
    let preview_x = preview_viewport.x.saturating_add(horizontal_offset);
    let preview_y = preview_viewport.y.saturating_add(vertical_offset);

    render_directory_buffer(
        mode,
        frame,
        parent_viewport,
        parent_buffer,
        parent_x,
        parent_y,
    );

    buffer_view(
        current_viewport,
        mode,
        &current_buffer.buffer,
        frame,
        current_x,
        current_y,
    );

    match preview_buffer {
        PreviewView::Directory(buffer) => {
            render_directory_buffer(mode, frame, preview_viewport, buffer, preview_x, preview_y);
        }
        PreviewView::Image(buffer) => {
            let rect = Rect {
                x: preview_x,
                y: preview_y,
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
                preview_x,
                preview_y,
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
