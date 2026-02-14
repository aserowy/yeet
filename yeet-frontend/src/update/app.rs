use std::cmp::Ordering;

use yeet_buffer::model::viewport::ViewPort;

use crate::model::{App, Buffer, Window};

pub fn get_focused(app: &App) -> (&ViewPort, &Buffer) {
    let (viewport, focused_id) = match &app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(_, vp, _) => (vp, &vp.buffer_id),
    };

    match app.buffers.get(focused_id) {
        Some(it) => (viewport, it),
        None => todo!(),
    }
}

#[allow(dead_code)]
pub fn focused_window_mut(app: &mut App) -> (&mut ViewPort, usize) {
    match &mut app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(_, vp, _) => {
            let id = vp.buffer_id;
            (vp, id)
        }
    }
}

#[allow(dead_code)]
pub fn focused_id(app: &App) -> usize {
    match &app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(_, vp, _) => vp.buffer_id,
    }
}

pub fn get_focused_mut(app: &mut App) -> (&mut ViewPort, &mut Buffer) {
    let (vp, focused_id) = match &mut app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(_, vp, _) => {
            let id = vp.buffer_id;
            (vp, id)
        }
    };

    match app.buffers.get_mut(&focused_id) {
        Some(it) => (vp, it),
        None => todo!(),
    }
}

pub fn get_next_buffer_id(app: &mut App) -> usize {
    let mut next_id = if app.latest_buffer_id >= 9999 {
        1
    } else {
        app.latest_buffer_id + 1
    };

    let mut running_ids: Vec<_> = app.buffers.keys().collect();
    running_ids.sort();

    for id in running_ids {
        match next_id.cmp(id) {
            Ordering::Equal => next_id += 1,
            Ordering::Greater => break,
            Ordering::Less => {}
        }
    }

    app.latest_buffer_id = next_id;

    next_id
}

pub fn set_focused_buffer(app: &mut App, id: usize) {
    match &mut app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(_, vp, _) => vp.buffer_id = id,
    };
}

pub fn directory_viewports(app: &App) -> (&ViewPort, &ViewPort, &ViewPort) {
    match &app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(parent, current, preview) => (parent, current, preview),
    }
}

pub fn directory_viewports_mut(app: &mut App) -> (&mut ViewPort, &mut ViewPort, &mut ViewPort) {
    match &mut app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(parent, current, preview) => (parent, current, preview),
    }
}

pub fn directory_buffer_ids(app: &App) -> (usize, usize, usize) {
    let (parent, current, preview) = directory_viewports(app);
    (parent.buffer_id, current.buffer_id, preview.buffer_id)
}

pub fn directory_buffers(app: &App) -> (&Buffer, &Buffer, &Buffer) {
    let (parent_id, current_id, preview_id) = directory_buffer_ids(app);
    if parent_id == current_id || parent_id == preview_id || current_id == preview_id {
        todo!()
    }

    let parent = app.buffers.get(&parent_id).expect("parent buffer");
    let current = app.buffers.get(&current_id).expect("current buffer");
    let preview = app.buffers.get(&preview_id).expect("preview buffer");

    (parent, current, preview)
}
