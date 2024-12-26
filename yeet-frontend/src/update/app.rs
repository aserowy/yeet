use std::{cmp::Ordering, mem};

use crate::model::{App, Buffer, Window};

pub fn get_focused_mut(app: &mut App) -> &mut Buffer {
    let focused_id = match app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Content(_, _, it) => it,
    };

    match app.buffers.get_mut(&focused_id) {
        Some(it) => return it,
        None => todo!(),
    };
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
        match next_id.cmp(&id) {
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
        Window::Content(_, _, it) => mem::replace(it, id),
    };
}
