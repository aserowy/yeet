use std::{cmp::Ordering, path::Path};

use yeet_buffer::model::{viewport::ViewPort, Mode};

use crate::{
    action::Action,
    model::{App, Buffer, Window},
    update::cursor,
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

#[tracing::instrument(skip(app))]
pub fn get_or_create_directory_buffer(
    app: &mut App,
    path: &Path,
    selection: &Option<String>,
) -> (usize, Option<Action>) {
    let matching_ids: Vec<(usize, &'static str)> = app
        .buffers
        .iter()
        .filter_map(|(id, buffer)| match buffer {
            Buffer::Directory(it) if it.path == path => Some((*id, "Directory")),
            Buffer::Content(it) if it.path == path => Some((*id, "Content")),
            Buffer::Image(it) if it.path == path => Some((*id, "Image")),
            Buffer::PathReference(p) if p == path => Some((*id, "PathReference")),
            _ => None,
        })
        .collect();

    tracing::trace!(
        path = %path.display(),
        total_buffers = app.buffers.len(),
        matching_count = matching_ids.len(),
        "checking for existing buffer"
    );

    if matching_ids.len() > 1 {
        tracing::warn!(
            path = %path.display(),
            matching_ids = ?matching_ids,
            "detected multiple buffers with the same path"
        );
    }

    if let Some((id, buffer_type)) = matching_ids.first() {
        tracing::trace!(
            id = %id,
            buffer_type = %buffer_type,
            path = %path.display(),
            "found existing buffer"
        );

        if let Some(selection) = selection {
            let mut buffer = app.buffers.get_mut(id).expect("buffer should exist");
            if let Buffer::Directory(buffer) = &mut buffer {
                cursor::set_cursor_index_to_selection(
                    None,
                    &Mode::Normal,
                    &mut buffer.buffer,
                    selection,
                );
            }
        }

        return (*id, None);
    }

    let id = get_next_buffer_id(app);

    let existing_paths: Vec<_> = app
        .buffers
        .iter()
        .filter_map(|(buf_id, buffer)| {
            let path_str = match buffer {
                Buffer::Directory(it) => Some(format!("{}:Dir:{}", buf_id, it.path.display())),
                Buffer::Content(it) => Some(format!("{}:Content:{}", buf_id, it.path.display())),
                Buffer::Image(it) => Some(format!("{}:Image:{}", buf_id, it.path.display())),
                Buffer::PathReference(p) => Some(format!("{}:PathRef:{}", buf_id, p.display())),
                Buffer::Empty => None,
            };
            path_str
        })
        .collect();

    tracing::debug!(
        id = %id,
        path = %path.display(),
        total_buffers = app.buffers.len(),
        existing_buffers = ?existing_paths,
        "created new buffer"
    );

    app.buffers
        .insert(id, Buffer::PathReference(path.to_path_buf()));

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
