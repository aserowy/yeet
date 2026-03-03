use std::{cmp::Ordering, path::Path};

use yeet_buffer::model::viewport::ViewPort;

use crate::{
    action::Action,
    model::{App, Buffer, Contents, Window},
};

pub fn get_focused_current_mut(app: &mut App) -> (&mut ViewPort, &mut Buffer) {
    let (vp, focused_id) = match &mut app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(_, vp, _) => {
            let id = vp.buffer_id;
            (vp, id)
        }
    };

    match app.contents.buffers.get_mut(&focused_id) {
        Some(it) => (vp, it),
        None => todo!(),
    }
}

pub fn get_focused_directory_viewports(app: &App) -> (&ViewPort, &ViewPort, &ViewPort) {
    match &app.window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(parent, current, preview) => (parent, current, preview),
    }
}

pub fn get_focused_directory_viewports_mut(
    window: &mut Window,
) -> (&mut ViewPort, &mut ViewPort, &mut ViewPort) {
    match window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(parent, current, preview) => (parent, current, preview),
    }
}

pub fn get_focused_directory_buffer_ids(app: &App) -> (usize, usize, usize) {
    let (parent, current, preview) = get_focused_directory_viewports(app);
    (parent.buffer_id, current.buffer_id, preview.buffer_id)
}

pub fn get_viewport_by_buffer_id_mut(
    window: &mut Window,
    buffer_id: usize,
) -> Option<&mut ViewPort> {
    match window {
        Window::Horizontal(_, _) => todo!(),
        Window::Directory(parent, current, preview) => {
            if parent.buffer_id == buffer_id {
                Some(parent)
            } else if current.buffer_id == buffer_id {
                Some(current)
            } else if preview.buffer_id == buffer_id {
                Some(preview)
            } else {
                None
            }
        }
    }
}

#[tracing::instrument(skip(contents))]
pub fn resolve_buffer(
    contents: &mut Contents,
    path: &Path,
    selection: &Option<String>,
) -> (usize, Option<Action>) {
    let matching_ids: Vec<(usize, &'static str)> = contents
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
        total_buffers = contents.buffers.len(),
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

        return (*id, None);
    }

    let id = get_next_buffer_id(contents);

    let existing_paths: Vec<_> = contents
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
        total_buffers = contents.buffers.len(),
        existing_buffers = ?existing_paths,
        "created new buffer"
    );

    contents
        .buffers
        .insert(id, Buffer::PathReference(path.to_path_buf()));

    (
        id,
        Some(Action::Load(path.to_path_buf(), selection.clone())),
    )
}

pub fn get_empty_buffer(contents: &mut Contents) -> usize {
    let existing_id = contents
        .buffers
        .iter()
        .find_map(|(id, buffer)| match buffer {
            Buffer::Empty => Some(*id),
            _ => None,
        });

    if let Some(id) = existing_id {
        return id;
    }
    let id = get_next_buffer_id(contents);
    contents.buffers.insert(id, Buffer::Empty);
    id
}

pub fn get_next_buffer_id(contents: &mut Contents) -> usize {
    let mut next_id = if contents.latest_buffer_id >= 9999 {
        1
    } else {
        contents.latest_buffer_id + 1
    };

    let mut running_ids: Vec<_> = contents.buffers.keys().collect();
    running_ids.sort();

    for id in running_ids {
        match next_id.cmp(id) {
            Ordering::Equal => next_id += 1,
            Ordering::Greater => break,
            Ordering::Less => {}
        }
    }

    contents.latest_buffer_id = next_id;

    next_id
}
