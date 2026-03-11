use std::{cmp::Ordering, path::Path};

use yeet_buffer::model::viewport::ViewPort;

use crate::{
    action::Action,
    error::AppError,
    model::{App, Buffer, Contents, SplitFocus, Window},
};

pub fn get_focused_current_mut<'a>(
    window: &'a mut Window,
    contents: &'a mut Contents,
) -> Result<(&'a mut ViewPort, &'a mut Buffer), AppError> {
    let vp = window.focused_viewport_mut();
    let focused_id = vp.buffer_id;

    match contents.buffers.get_mut(&focused_id) {
        Some(it) => Ok((vp, it)),
        None => Err(AppError::BufferNotFound(focused_id)),
    }
}

pub fn get_focused_directory_viewports(
    window: &Window,
) -> Option<(&ViewPort, &ViewPort, &ViewPort)> {
    match window {
        Window::Horizontal {
            first,
            second,
            focus,
        }
        | Window::Vertical {
            first,
            second,
            focus,
        } => match focus {
            SplitFocus::First => get_focused_directory_viewports(first),
            SplitFocus::Second => get_focused_directory_viewports(second),
        },
        Window::Directory(parent, current, preview) => Some((parent, current, preview)),
        Window::Tasks(_) => None,
    }
}

pub fn get_focused_directory_viewports_mut(
    window: &mut Window,
) -> Option<(&mut ViewPort, &mut ViewPort, &mut ViewPort)> {
    match window {
        Window::Horizontal {
            first,
            second,
            focus,
        }
        | Window::Vertical {
            first,
            second,
            focus,
        } => match focus {
            SplitFocus::First => get_focused_directory_viewports_mut(first),
            SplitFocus::Second => get_focused_directory_viewports_mut(second),
        },
        Window::Directory(parent, current, preview) => Some((parent, current, preview)),
        Window::Tasks(_) => None,
    }
}

pub fn get_focused_directory_buffer_ids(window: &Window) -> Option<(usize, usize, usize)> {
    let (parent, current, preview) = get_focused_directory_viewports(window)?;
    Some((parent.buffer_id, current.buffer_id, preview.buffer_id))
}

pub fn get_viewport_by_buffer_id_mut(
    window: &mut Window,
    buffer_id: usize,
) -> Option<&mut ViewPort> {
    match window {
        Window::Horizontal { first, second, .. } | Window::Vertical { first, second, .. } => {
            get_viewport_by_buffer_id_mut(first, buffer_id)
                .or_else(|| get_viewport_by_buffer_id_mut(second, buffer_id))
        }
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
        Window::Tasks(vp) => {
            if vp.buffer_id == buffer_id {
                Some(vp)
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
                Buffer::Tasks(_) => None,
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

pub fn get_buffer_path(app: &App, buffer_id: usize) -> Result<Option<&Path>, AppError> {
    let buffer = app.contents.buffers.get(&buffer_id);

    match buffer {
        Some(buffer) => Ok(buffer.resolve_path()),
        None => Err(AppError::BufferNotFound(buffer_id)),
    }
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use yeet_buffer::model::viewport::ViewPort;

    use crate::model::{App, Buffer, Contents, SplitFocus, TasksBuffer, Window};

    use super::*;

    fn make_horizontal_app() -> App {
        let mut buffers = HashMap::new();
        buffers.insert(10, Buffer::Empty);
        buffers.insert(11, Buffer::Empty);
        buffers.insert(12, Buffer::Empty);
        buffers.insert(20, Buffer::Tasks(TasksBuffer::default()));

        let mut tabs = HashMap::new();
        tabs.insert(
            1,
            Window::Horizontal {
                first: Box::new(Window::Directory(
                    ViewPort {
                        buffer_id: 10,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 11,
                        ..Default::default()
                    },
                    ViewPort {
                        buffer_id: 12,
                        ..Default::default()
                    },
                )),
                second: Box::new(Window::Tasks(ViewPort {
                    buffer_id: 20,
                    ..Default::default()
                })),
                focus: SplitFocus::Second,
            },
        );

        App {
            commandline: Default::default(),
            contents: Contents {
                buffers,
                latest_buffer_id: 20,
            },
            tabs,
            current_tab_id: 1,
        }
    }

    #[test]
    fn get_focused_current_mut_returns_tasks_when_focused() {
        let mut app = make_horizontal_app();
        let (window, contents) = app
            .current_window_and_contents_mut()
            .expect("test requires current tab");

        let (vp, buffer) = get_focused_current_mut(window, contents)
            .expect("focused viewport should have a valid buffer");

        assert_eq!(vp.buffer_id, 20);
        assert!(matches!(buffer, Buffer::Tasks(_)));
    }

    #[test]
    fn get_focused_directory_viewports_none_for_tasks() {
        let app = make_horizontal_app();
        let window = app.current_window().expect("test requires current tab");
        assert!(get_focused_directory_viewports(window).is_none());
    }

    #[test]
    fn get_focused_directory_viewports_some_through_horizontal() {
        let mut app = make_horizontal_app();
        let window = app.current_window_mut().expect("test requires current tab");
        if let Window::Horizontal { focus, .. } = window {
            *focus = SplitFocus::First;
        }
        let window = app.current_window().expect("test requires current tab");
        let result = get_focused_directory_viewports(window);
        assert!(result.is_some());
        let (parent, current, preview) = result.unwrap();
        assert_eq!(parent.buffer_id, 10);
        assert_eq!(current.buffer_id, 11);
        assert_eq!(preview.buffer_id, 12);
    }

    #[test]
    fn get_focused_directory_buffer_ids_none_for_tasks() {
        let app = make_horizontal_app();
        let window = app.current_window().expect("test requires current tab");
        assert!(get_focused_directory_buffer_ids(window).is_none());
    }

    #[test]
    fn get_viewport_by_buffer_id_mut_finds_in_second_child() {
        let mut app = make_horizontal_app();
        let window = app.current_window_mut().expect("test requires current tab");
        let vp = get_viewport_by_buffer_id_mut(window, 20);
        assert!(vp.is_some());
        assert_eq!(vp.unwrap().buffer_id, 20);
    }

    #[test]
    fn get_viewport_by_buffer_id_mut_finds_in_first_child() {
        let mut app = make_horizontal_app();
        let window = app.current_window_mut().expect("test requires current tab");
        let vp = get_viewport_by_buffer_id_mut(window, 11);
        assert!(vp.is_some());
        assert_eq!(vp.unwrap().buffer_id, 11);
    }

    #[test]
    fn get_viewport_by_buffer_id_mut_returns_none_for_missing() {
        let mut app = make_horizontal_app();
        let window = app.current_window_mut().expect("test requires current tab");
        assert!(get_viewport_by_buffer_id_mut(window, 999).is_none());
    }
}
