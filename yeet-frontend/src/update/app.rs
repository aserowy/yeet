use std::{cmp::Ordering, path::Path};

use yeet_buffer::model::viewport::ViewPort;

use crate::{
    action::Action,
    model::{App, Buffer, DirectoryBuffer, Window},
};

pub fn get_focused_current_mut(app: &mut App) -> (&mut ViewPort, &mut Buffer) {
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

pub fn get_or_create_directory_buffer(
    app: &mut App,
    path: &Path,
    selection: &Option<String>,
) -> (usize, Option<Action>) {
    let existing_id = app.buffers.iter().find_map(|(id, buffer)| match buffer {
        Buffer::Directory(it) if it.path == path => Some(*id),
        Buffer::Content(it) if it.path == path => Some(*id),
        Buffer::Image(it) if it.path == path => Some(*id),
        _ => None,
    });

    if let Some(id) = existing_id {
        return (id, None);
    }

    let id = get_next_buffer_id(app);
    app.buffers.insert(
        id,
        Buffer::Directory(DirectoryBuffer {
            path: path.to_path_buf(),
            ..Default::default()
        }),
    );
    (
        id,
        Some(Action::Load(path.to_path_buf(), selection.clone())),
    )
}

pub fn create_empty_buffer(app: &mut App) -> usize {
    let existing_id = app.buffers.iter().find_map(|(id, buffer)| match buffer {
        Buffer::Empty => Some(*id),
        _ => None,
    });

    if let Some(id) = existing_id {
        return id;
    }
    let id = get_next_buffer_id(app);
    app.buffers.insert(id, Buffer::Empty);
    id
}
