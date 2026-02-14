use std::{cmp::Ordering, mem};

use yeet_buffer::model::viewport::ViewPort;

use crate::model::{App, Buffer, Window};

pub fn get_focused(app: &App) -> (&ViewPort, &Buffer) {
    let (viewport, focused_id) = match &app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(vp, it) => (vp, it),
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
        Window::Content(vp, it) => (vp, *it),
    }
}

#[allow(dead_code)]
pub fn focused_id(app: &App) -> usize {
    match &app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(_, id) => *id,
    }
}

pub fn get_focused_mut(app: &mut App) -> (&mut ViewPort, &mut Buffer) {
    let (vp, focused_id) = match &mut app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(vp, it) => (vp, it),
    };

    match app.buffers.get_mut(focused_id) {
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
        Window::Content(_, it) => mem::replace(it, id),
    };
}
