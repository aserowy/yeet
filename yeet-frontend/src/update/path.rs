use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    slice,
};

use yeet_buffer::{message::BufferMessage, model::viewport::ViewPort, model::Mode};

use crate::{
    action::Action,
    error::AppError,
    model::{
        history::History,
        junkyard::JunkYard,
        mark::{Marks, MARK_SIGN_ID},
        qfix::{QuickFix, QFIX_SIGN_ID},
        App, Buffer, Contents, Window,
    },
    theme::Theme,
    update::{app, cursor, selection},
};

use super::{enumeration, history, junkyard::remove_from_junkyard, sign};

#[tracing::instrument(skip(app, theme))]
pub fn add(
    history: &mut History,
    marks: &Marks,
    qfix: &QuickFix,
    mode: &Mode,
    app: &mut App,
    paths: &[PathBuf],
    theme: &Theme,
) -> Result<Vec<Action>, AppError> {
    let mut actions = Vec::new();
    for path in paths {
        actions.extend(update_directory_buffers_on_add(history, mode, app, path, theme));
    }

    let marked_paths: Vec<_> = paths
        .iter()
        .filter(|path| marks.entries.values().any(|marked| marked == *path))
        .cloned()
        .collect();
    if !marked_paths.is_empty() {
        sign::set_sign_for_paths(
            app.contents.buffers.values_mut().collect(),
            marked_paths,
            MARK_SIGN_ID,
        );
    }

    let qfix_paths: Vec<_> = paths
        .iter()
        .filter(|path| qfix.entries.contains(*path))
        .cloned()
        .collect();
    if !qfix_paths.is_empty() {
        sign::set_sign_for_paths(
            app.contents.buffers.values_mut().collect(),
            qfix_paths,
            QFIX_SIGN_ID,
        );
    }

    Ok(actions)
}

#[tracing::instrument(skip(junk, app))]
pub fn remove(
    history: &mut History,
    marks: &mut Marks,
    qfix: &mut QuickFix,
    junk: &mut JunkYard,
    mode: &Mode,
    app: &mut App,
    path: &Path,
) -> Result<Vec<Action>, AppError> {
    if path.starts_with(junk.path.clone()) {
        remove_from_junkyard(junk, path);
    }

    history::remove_entry(history, path);

    let mut actions = update_directory_buffers_on_remove(history, mode, app, path)?;
    actions.extend(cleanup_removed_buffers(history, mode, app, path));
    let removed_marks = remove_marks_for_path(marks, path);
    if !removed_marks.is_empty() {
        sign::unset_sign_for_paths(
            app.contents.buffers.values_mut().collect(),
            removed_marks,
            MARK_SIGN_ID,
        );
    }

    let removed_qfix: Vec<_> = qfix
        .entries
        .iter()
        .filter(|entry| entry.starts_with(path))
        .cloned()
        .collect();

    sign::unset_sign_for_paths(
        app.contents.buffers.values_mut().collect(),
        removed_qfix,
        QFIX_SIGN_ID,
    );

    Ok(actions)
}

fn remove_marks_for_path(marks: &mut Marks, path: &Path) -> Vec<PathBuf> {
    let mut removed_paths = Vec::new();
    let mut marks_to_remove = Vec::new();
    for (mark, mark_path) in marks.entries.iter() {
        if mark_path.starts_with(path) {
            removed_paths.push(mark_path.clone());
            marks_to_remove.push(*mark);
        }
    }
    for mark in marks_to_remove {
        marks.entries.remove(&mark);
    }
    removed_paths
}

fn update_directory_buffers_on_add(
    history: &mut History,
    mode: &Mode,
    app: &mut App,
    path: &Path,
    theme: &Theme,
) -> Vec<Action> {
    let (parent, name) = match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => (parent, name.to_string_lossy().to_string()),
        _ => return Vec::new(),
    };

    let target_buffer_ids: HashSet<usize> = app
        .contents
        .buffers
        .iter()
        .filter_map(|(id, buffer)| match buffer {
            Buffer::Directory(dir) if dir.path == parent => Some(*id),
            _ => None,
        })
        .collect();

    let selection_by_viewport = collect_viewport_selections_for_buffers(app, &target_buffer_ids);

    let mut updated_buffers = Vec::new();
    let (tabs, contents) = (&mut app.tabs, &mut app.contents);
    for window in tabs.values_mut() {
        for (buffer_id, buffer) in contents.buffers.iter_mut() {
            let Buffer::Directory(dir) = buffer else {
                continue;
            };

            if dir.path != parent {
                continue;
            }

            let viewport = app::get_viewport_by_buffer_id_mut(window, *buffer_id);
            if dir
                .buffer
                .lines
                .iter()
                .any(|line| line.content.to_stripped_string() == name)
            {
                yeet_buffer::update(
                    viewport,
                    mode,
                    &mut dir.buffer,
                    slice::from_ref(&BufferMessage::SortContent(super::SORT)),
                );

                updated_buffers.push(*buffer_id);

                continue;
            }

            let added_existing_directory = dir.buffer.lines.iter().position(|line| {
                line.content
                    .to_stripped_string()
                    .starts_with(&format!("{name}/"))
            });

            let kind = if path.is_dir() {
                crate::event::ContentKind::Directory
            } else {
                crate::event::ContentKind::File
            };

            let bufferline = enumeration::from_enumeration(&name, &kind, theme);
            if let Some(index) = added_existing_directory {
                if let Some(line) = dir.buffer.lines.get_mut(index) {
                    *line = bufferline;
                }

                yeet_buffer::update(
                    viewport,
                    mode,
                    &mut dir.buffer,
                    std::slice::from_ref(&BufferMessage::SortContent(super::SORT)),
                );
            } else {
                yeet_buffer::update(
                    viewport,
                    mode,
                    &mut dir.buffer,
                    std::slice::from_ref(&BufferMessage::AddLine(bufferline, super::SORT)),
                );
            }
            updated_buffers.push(*buffer_id);
        }
    }

    update_viewports_for_buffers(history, app, mode, &updated_buffers, &selection_by_viewport)
}

fn update_directory_buffers_on_remove(
    history: &mut History,
    mode: &Mode,
    app: &mut App,
    path: &Path,
) -> Result<Vec<Action>, AppError> {
    let parent_name = match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => Some((parent, name.to_string_lossy().to_string())),
        _ => None,
    };

    let actions = if let Some((parent, name)) = parent_name {
        let target_buffer_ids: HashSet<usize> = app
            .contents
            .buffers
            .iter()
            .filter_map(|(id, buffer)| match buffer {
                Buffer::Directory(dir) if dir.path == parent => Some(*id),
                _ => None,
            })
            .collect();
        let selection_by_viewport =
            collect_viewport_selections_for_buffers(app, &target_buffer_ids);

        let mut updated_buffers = Vec::new();
        let (tabs, contents) = (&mut app.tabs, &mut app.contents);
        for window in tabs.values_mut() {
            for (buffer_id, buffer) in contents.buffers.iter_mut() {
                let Buffer::Directory(dir) = buffer else {
                    continue;
                };

                if dir.path != parent {
                    continue;
                }

                let mut indices: Vec<usize> = dir
                    .buffer
                    .lines
                    .iter()
                    .enumerate()
                    .filter_map(|(index, line)| {
                        if line.content.to_stripped_string() == name {
                            Some(index)
                        } else {
                            None
                        }
                    })
                    .collect();

                if indices.is_empty() {
                    continue;
                }

                let mut viewport = app::get_viewport_by_buffer_id_mut(window, *buffer_id);
                indices.sort_unstable_by(|a, b| b.cmp(a));
                for index in indices {
                    yeet_buffer::update(
                        viewport.as_deref_mut(),
                        mode,
                        &mut dir.buffer,
                        slice::from_ref(&BufferMessage::RemoveLine(index)),
                    );
                }
                updated_buffers.push(*buffer_id);
            }
        }

        update_viewports_for_buffers(history, app, mode, &updated_buffers, &selection_by_viewport)
    } else {
        Vec::new()
    };

    Ok(actions)
}

fn cleanup_removed_buffers(
    history: &mut History,
    mode: &Mode,
    app: &mut App,
    removed: &Path,
) -> Vec<Action> {
    let mut actions = Vec::new();
    let (tabs, contents) = (&mut app.tabs, &mut app.contents);
    for window in tabs.values_mut() {
        actions.extend(cleanup_removed_buffers_in_window(
            history, mode, window, contents, removed,
        ));
    }

    prune_removed_buffers(app, removed);

    actions
}

fn cleanup_removed_buffers_in_window(
    history: &mut History,
    mode: &Mode,
    window: &mut Window,
    contents: &mut Contents,
    removed: &Path,
) -> Vec<Action> {
    let mut actions = Vec::new();
    match window {
        Window::Horizontal { first, second, .. } | Window::Vertical { first, second, .. } => {
            actions.extend(cleanup_removed_buffers_in_window(
                history, mode, first, contents, removed,
            ));
            actions.extend(cleanup_removed_buffers_in_window(
                history, mode, second, contents, removed,
            ));
        }
        Window::Directory(parent, current, preview) => {
            let old_preview_path = contents
                .buffers
                .get(&preview.buffer_id)
                .and_then(|buffer| buffer.resolve_path())
                .map(|path| path.to_path_buf());

            let mut refresh_preview = relocate_viewport_if_removed(
                history,
                mode,
                contents,
                parent,
                removed,
                &mut actions,
            );
            refresh_preview |= relocate_viewport_if_removed(
                history,
                mode,
                contents,
                current,
                removed,
                &mut actions,
            );

            if contents
                .buffers
                .get(&preview.buffer_id)
                .and_then(|buffer| buffer.resolve_path())
                .map(|path| path.starts_with(removed))
                .unwrap_or(false)
            {
                refresh_preview = true;
            }

            if refresh_preview {
                let current_is_directory = matches!(
                    contents.buffers.get(&current.buffer_id),
                    Some(Buffer::Directory(_))
                );
                if !current_is_directory {
                    actions.extend(selection::set_preview_buffer_for_selection(
                        window, contents, history, None,
                    ));
                    return actions;
                }

                actions.extend(
                    selection::refresh_preview_from_selection(
                        history,
                        window,
                        contents,
                        old_preview_path,
                    )
                    .unwrap_or_default(),
                );
            }
        }
        Window::Tasks(_) => {}
    }

    actions
}

fn relocate_viewport_if_removed(
    history: &mut History,
    mode: &Mode,
    contents: &mut Contents,
    viewport: &mut ViewPort,
    removed: &Path,
    actions: &mut Vec<Action>,
) -> bool {
    let path = match contents
        .buffers
        .get(&viewport.buffer_id)
        .and_then(|buffer| buffer.resolve_path())
    {
        Some(path) if path.starts_with(removed) => path.to_path_buf(),
        _ => return false,
    };

    let replacement = nearest_existing_ancestor(&path);
    let selection = history::selection(history, &replacement).map(|s| s.to_owned());
    let (new_id, load) = app::resolve_buffer(contents, &replacement, &selection);
    if let Some(load) = load {
        actions.push(load);
    }

    viewport.buffer_id = new_id;
    viewport.cursor = Default::default();
    let _ = cursor::set_index(contents, history, viewport, mode, selection.as_deref());

    true
}

fn nearest_existing_ancestor(path: &Path) -> PathBuf {
    for ancestor in path.ancestors() {
        if ancestor.exists() {
            return ancestor.to_path_buf();
        }
    }

    PathBuf::new()
}

fn prune_removed_buffers(app: &mut App, removed: &Path) {
    let mut referenced_ids = HashSet::new();
    for window in app.tabs.values() {
        referenced_ids.extend(window.buffer_ids());
    }

    let removed_ids: Vec<usize> = app
        .contents
        .buffers
        .iter()
        .filter_map(|(id, buffer)| {
            if referenced_ids.contains(id) {
                return None;
            }
            buffer
                .resolve_path()
                .filter(|path| path.starts_with(removed))
                .map(|_| *id)
        })
        .collect();

    for id in removed_ids {
        app.contents.buffers.remove(&id);
    }
}

fn update_viewports_for_buffers(
    history: &mut History,
    app: &mut App,
    mode: &Mode,
    buffer_ids: &[usize],
    selection_by_viewport: &HashMap<*const ViewPort, Option<String>>,
) -> Vec<Action> {
    if buffer_ids.is_empty() {
        return Vec::new();
    }

    let mut target_ids: Vec<usize> = buffer_ids.to_vec();
    target_ids.sort_unstable();
    target_ids.dedup();

    let contents = &mut app.contents;
    let mut actions = Vec::new();
    for window in app.tabs.values_mut() {
        actions.extend(update_viewports_for_buffers_in_window(
            history,
            window,
            contents,
            mode,
            &target_ids,
            selection_by_viewport,
        ));
    }
    actions
}

fn update_viewports_for_buffers_in_window(
    history: &mut History,
    window: &mut Window,
    contents: &mut Contents,
    mode: &Mode,
    buffer_ids: &[usize],
    selection_by_viewport: &HashMap<*const ViewPort, Option<String>>,
) -> Vec<Action> {
    let mut actions = Vec::new();
    match window {
        Window::Horizontal { first, second, .. } | Window::Vertical { first, second, .. } => {
            actions.extend(update_viewports_for_buffers_in_window(
                history,
                first,
                contents,
                mode,
                buffer_ids,
                selection_by_viewport,
            ));
            actions.extend(update_viewports_for_buffers_in_window(
                history,
                second,
                contents,
                mode,
                buffer_ids,
                selection_by_viewport,
            ));
        }
        Window::Directory(parent, current, preview) => {
            update_viewport_for_buffer(parent, contents, mode, buffer_ids, selection_by_viewport);
            update_viewport_for_buffer(current, contents, mode, buffer_ids, selection_by_viewport);
            update_viewport_for_buffer(preview, contents, mode, buffer_ids, selection_by_viewport);

            let selection = contents
                .buffers
                .get(&preview.buffer_id)
                .and_then(|buffer| buffer.resolve_path().map(|path| path.to_path_buf()));

            actions.extend(
                selection::refresh_preview_from_selection(history, window, contents, selection)
                    .unwrap_or_default(),
            );
        }
        Window::Tasks(viewport) => {
            update_viewport_for_buffer(viewport, contents, mode, buffer_ids, selection_by_viewport);
        }
    };

    actions
}

fn collect_viewport_selections_for_buffers(
    app: &App,
    buffer_ids: &HashSet<usize>,
) -> HashMap<*const ViewPort, Option<String>> {
    let mut selections = HashMap::new();
    for window in app.tabs.values() {
        collect_viewport_selections_for_buffers_in_window(
            window,
            &app.contents,
            buffer_ids,
            &mut selections,
        );
    }
    selections
}

fn collect_viewport_selections_for_buffers_in_window(
    window: &Window,
    contents: &crate::model::Contents,
    buffer_ids: &HashSet<usize>,
    selections: &mut HashMap<*const ViewPort, Option<String>>,
) {
    match window {
        Window::Horizontal { first, second, .. } | Window::Vertical { first, second, .. } => {
            collect_viewport_selections_for_buffers_in_window(
                first, contents, buffer_ids, selections,
            );
            collect_viewport_selections_for_buffers_in_window(
                second, contents, buffer_ids, selections,
            );
        }
        Window::Directory(parent, current, preview) => {
            collect_viewport_selection(parent, contents, buffer_ids, selections);
            collect_viewport_selection(current, contents, buffer_ids, selections);
            collect_viewport_selection(preview, contents, buffer_ids, selections);
        }
        Window::Tasks(viewport) => {
            collect_viewport_selection(viewport, contents, buffer_ids, selections);
        }
    }
}

fn collect_viewport_selection(
    viewport: &ViewPort,
    contents: &crate::model::Contents,
    buffer_ids: &HashSet<usize>,
    selections: &mut HashMap<*const ViewPort, Option<String>>,
) {
    if !buffer_ids.contains(&viewport.buffer_id) {
        return;
    }

    let selection = match contents.buffers.get(&viewport.buffer_id) {
        Some(Buffer::Directory(dir)) => dir
            .buffer
            .lines
            .get(viewport.cursor.vertical_index)
            .map(|line| line.content.to_stripped_string()),
        Some(Buffer::Content(content)) => content
            .buffer
            .lines
            .get(viewport.cursor.vertical_index)
            .map(|line| line.content.to_stripped_string()),
        Some(Buffer::Tasks(tasks)) => tasks
            .buffer
            .lines
            .get(viewport.cursor.vertical_index)
            .map(|line| line.content.to_stripped_string()),
        _ => None,
    };

    selections.insert(viewport as *const _, selection);
}

fn update_viewport_for_buffer(
    viewport: &mut ViewPort,
    contents: &Contents,
    mode: &Mode,
    buffer_ids: &[usize],
    selection_by_viewport: &HashMap<*const ViewPort, Option<String>>,
) {
    if !buffer_ids.contains(&viewport.buffer_id) {
        return;
    }

    let Some(buffer) = contents.buffers.get(&viewport.buffer_id) else {
        return;
    };

    let selection = selection_by_viewport
        .get(&(viewport as *const _))
        .cloned()
        .unwrap_or(None);

    match buffer {
        Buffer::Directory(dir) => {
            update_directory_viewport_selection(viewport, mode, &dir.buffer, selection);
        }
        Buffer::Content(content) => {
            update_directory_viewport_selection(viewport, mode, &content.buffer, selection);
        }
        Buffer::Tasks(tasks) => {
            update_directory_viewport_selection(viewport, mode, &tasks.buffer, selection);
        }
        Buffer::Image(_) | Buffer::PathReference(_) | Buffer::Empty => {}
    }
}

fn update_directory_viewport_selection(
    viewport: &mut ViewPort,
    mode: &Mode,
    buffer: &yeet_buffer::model::TextBuffer,
    selection: Option<String>,
) {
    if let Some(selection) = selection {
        if let Some(index) = buffer
            .lines
            .iter()
            .position(|line| line.content.to_stripped_string() == selection)
        {
            viewport.cursor.vertical_index = index;
        }
    }

    yeet_buffer::update_viewport_by_buffer(viewport, mode, buffer);
}

#[cfg(test)]
mod test {
    use std::{fs, time::SystemTime};

    use yeet_buffer::model::{ansi::Ansi, viewport::ViewPort, BufferLine, Mode, TextBuffer};

    use crate::model::{
        history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, App, ContentBuffer,
        DirectoryBuffer, Window,
    };
    use crate::update::app;

    use super::*;

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("yeet-path-test-{}", nanos))
    }

    fn insert_tab_with_directory(app: &mut App, tab_id: usize, path: &Path, lines: Vec<String>) {
        let parent_id = app::get_next_buffer_id(&mut app.contents);
        let current_id = app::get_next_buffer_id(&mut app.contents);
        let preview_id = app::get_next_buffer_id(&mut app.contents);

        app.contents.buffers.insert(
            parent_id,
            Buffer::Directory(DirectoryBuffer {
                path: path.to_path_buf(),
                ..Default::default()
            }),
        );
        app.contents.buffers.insert(
            current_id,
            Buffer::Directory(DirectoryBuffer {
                path: path.to_path_buf(),
                buffer: TextBuffer::from_lines(
                    lines
                        .into_iter()
                        .map(|content| BufferLine {
                            content: Ansi::new(&content),
                            ..Default::default()
                        })
                        .collect(),
                ),
                ..Default::default()
            }),
        );
        app.contents.buffers.insert(preview_id, Buffer::Empty);

        app.tabs.insert(
            tab_id,
            Window::Directory(
                ViewPort {
                    buffer_id: parent_id,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: current_id,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: preview_id,
                    ..Default::default()
                },
            ),
        );
    }

    fn insert_shared_current_split(app: &mut App, path: &Path, lines: Vec<String>) -> usize {
        let shared_id = app::get_next_buffer_id(&mut app.contents);
        app.contents.buffers.insert(
            shared_id,
            Buffer::Directory(DirectoryBuffer {
                path: path.to_path_buf(),
                buffer: TextBuffer::from_lines(
                    lines
                        .into_iter()
                        .map(|content| BufferLine {
                            content: Ansi::new(&content),
                            ..Default::default()
                        })
                        .collect(),
                ),
                ..Default::default()
            }),
        );

        let parent_first = app::get_next_buffer_id(&mut app.contents);
        let preview_first = app::get_next_buffer_id(&mut app.contents);
        let parent_second = app::get_next_buffer_id(&mut app.contents);
        let preview_second = app::get_next_buffer_id(&mut app.contents);

        app.contents.buffers.insert(
            parent_first,
            Buffer::Directory(DirectoryBuffer {
                path: path.to_path_buf(),
                ..Default::default()
            }),
        );
        app.contents.buffers.insert(
            parent_second,
            Buffer::Directory(DirectoryBuffer {
                path: path.to_path_buf(),
                ..Default::default()
            }),
        );
        app.contents.buffers.insert(preview_first, Buffer::Empty);
        app.contents.buffers.insert(preview_second, Buffer::Empty);

        let window = Window::Horizontal {
            first: Box::new(Window::Directory(
                ViewPort {
                    buffer_id: parent_first,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: shared_id,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: preview_first,
                    ..Default::default()
                },
            )),
            second: Box::new(Window::Directory(
                ViewPort {
                    buffer_id: parent_second,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: shared_id,
                    ..Default::default()
                },
                ViewPort {
                    buffer_id: preview_second,
                    ..Default::default()
                },
            )),
            focus: crate::model::SplitFocus::First,
        };

        app.tabs.insert(1, window);
        app.current_tab_id = 1;

        shared_id
    }

    fn set_split_current_cursors(app: &mut App, first_index: usize, second_index: usize) {
        let window = app.current_window_mut().expect("current window");
        let (first, second) = match window {
            Window::Horizontal { first, second, .. } => (first.as_mut(), second.as_mut()),
            _ => panic!("expected horizontal split"),
        };

        if let Window::Directory(_, current, _) = first {
            current.cursor.vertical_index = first_index;
        }
        if let Window::Directory(_, current, _) = second {
            current.cursor.vertical_index = second_index;
        }
    }

    fn split_current_indices(app: &App) -> (usize, usize) {
        let window = app.current_window().expect("current window");
        let (first, second) = match window {
            Window::Horizontal { first, second, .. } => (first.as_ref(), second.as_ref()),
            _ => panic!("expected horizontal split"),
        };

        let first_index = match first {
            Window::Directory(_, current, _) => current.cursor.vertical_index,
            _ => panic!("expected directory window"),
        };
        let second_index = match second {
            Window::Directory(_, current, _) => current.cursor.vertical_index,
            _ => panic!("expected directory window"),
        };

        (first_index, second_index)
    }

    #[test]
    fn removal_relocates_current_to_existing_ancestor_and_prunes_buffers() {
        use crate::model::{DirectoryBuffer, Window};
        use yeet_buffer::model::viewport::ViewPort;

        let base = unique_temp_dir();
        let removed = base.join("removed");
        let nested = removed.join("inner");
        fs::create_dir_all(&nested).expect("create directories");

        let file_path = removed.join("file.txt");
        fs::write(&file_path, "content").expect("create file");

        let mut app = App::default();

        let parent_id = 1;
        let current_id = app::get_next_buffer_id(&mut app.contents);
        let preview_id = app::get_next_buffer_id(&mut app.contents);

        app.contents
            .buffers
            .insert(parent_id, Buffer::Directory(DirectoryBuffer::default()));
        app.contents.buffers.insert(
            current_id,
            Buffer::Directory(DirectoryBuffer {
                path: nested.clone(),
                ..Default::default()
            }),
        );
        app.contents.buffers.insert(
            preview_id,
            Buffer::Content(ContentBuffer {
                path: file_path.clone(),
                ..Default::default()
            }),
        );

        let window = app.current_window_mut().expect("test requires current tab");
        *window = Window::Directory(
            ViewPort {
                buffer_id: parent_id,
                ..Default::default()
            },
            ViewPort {
                buffer_id: current_id,
                ..Default::default()
            },
            ViewPort {
                buffer_id: preview_id,
                ..Default::default()
            },
        );

        let extra_id = app::get_next_buffer_id(&mut app.contents);
        app.contents.buffers.insert(
            extra_id,
            Buffer::Content(ContentBuffer {
                path: file_path.clone(),
                ..Default::default()
            }),
        );

        fs::remove_dir_all(&removed).expect("remove directory");

        let mut history = History::default();
        let mut marks = Marks::default();
        let mut qfix = QuickFix::default();
        let mut junk = JunkYard::default();

        let _ = remove(
            &mut history,
            &mut marks,
            &mut qfix,
            &mut junk,
            &Mode::Normal,
            &mut app,
            &removed,
        );

        let window = app.current_window().expect("test requires current tab");
        let (_, current_id, _) = app::get_focused_directory_buffer_ids(window).unwrap();
        let current_path = match app.contents.buffers.get(&current_id) {
            Some(Buffer::Directory(buffer)) => buffer.path.clone(),
            Some(Buffer::PathReference(path)) => path.clone(),
            _ => PathBuf::new(),
        };

        assert_eq!(current_path, base);
        assert!(app
            .contents
            .buffers
            .values()
            .filter_map(|b| b.resolve_path())
            .all(|path| !path.starts_with(&removed)));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn add_refreshes_preview_when_buffer_not_loaded_for_selection() {
        use yeet_buffer::model::Cursor;

        // Create a real temp directory so that path.exists() returns true.
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");

        let newfolder = base.join("newfolder");
        fs::create_dir_all(&newfolder).expect("create newfolder");

        let mut app = App::default();

        let parent_id = 1;
        let current_id = app::get_next_buffer_id(&mut app.contents);
        let preview_id = app::get_next_buffer_id(&mut app.contents);

        // Parent buffer: empty directory buffer (not important for this test).
        app.contents
            .buffers
            .insert(parent_id, Buffer::Directory(DirectoryBuffer::default()));

        // Current buffer: the base directory, containing "newfolder/" at cursor index 0.
        // The trailing slash simulates user-typed content from insert mode.
        app.contents.buffers.insert(
            current_id,
            Buffer::Directory(DirectoryBuffer {
                path: base.clone(),
                buffer: TextBuffer::from_lines(vec![BufferLine {
                    content: Ansi::new("newfolder/"),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
        );

        // Preview buffer: Empty — simulates that the preview was never loaded for
        // the newly-created folder (it didn't exist on disk when the cursor first
        // landed on it).
        app.contents.buffers.insert(preview_id, Buffer::Empty);

        let window = app.current_window_mut().expect("test requires current tab");
        *window = Window::Directory(
            ViewPort {
                buffer_id: parent_id,
                ..Default::default()
            },
            ViewPort {
                buffer_id: current_id,
                cursor: Cursor {
                    vertical_index: 0,
                    ..Default::default()
                },
                ..Default::default()
            },
            ViewPort {
                buffer_id: preview_id,
                ..Default::default()
            },
        );

        let mut history = History::default();
        let marks = Marks::default();
        let qfix = QuickFix::default();

        // Simulate PathsAdded for the new folder (path without trailing slash,
        // as the filesystem watcher reports it).
        let theme = Theme::default();
        let actions = add(
            &mut history,
            &marks,
            &qfix,
            &Mode::Navigation,
            &mut app,
            std::slice::from_ref(&newfolder),
            &theme,
        )
        .expect("path add must succeed");

        // The preview viewport must now point at a buffer for "newfolder", not
        // at the old Empty buffer. The buffer should be a PathReference (triggering
        // a Load action) or a Directory if already resolved.
        let window = app.current_window().expect("test requires current tab");
        let (_, _, new_preview_id) = app::get_focused_directory_buffer_ids(window).unwrap();
        let preview_buffer = app.contents.buffers.get(&new_preview_id);

        // The preview buffer should no longer be Empty.
        assert!(
            !matches!(preview_buffer, Some(Buffer::Empty)),
            "preview buffer should have been refreshed for the newly-created folder, \
             but it is still Empty (preview_id={})",
            new_preview_id,
        );

        // There should be a Load action for the new folder.
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::Load(p, _) if p == &newfolder)),
            "expected a Load action for {:?}, got: {:?}",
            newfolder,
            actions,
        );

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn preview_is_reset_when_removed_from_subtree() {
        use yeet_buffer::model::viewport::ViewPort;

        let base = unique_temp_dir();
        let keep = base.join("keep");
        let removed = base.join("removed");
        fs::create_dir_all(&keep).expect("create keep");
        fs::create_dir_all(&removed).expect("create removed");

        let removed_file = removed.join("gone.txt");
        fs::write(&removed_file, "content").expect("create file");

        let mut app = App::default();

        let parent_id = 1;
        let current_id = app::get_next_buffer_id(&mut app.contents);
        let preview_id = app::get_next_buffer_id(&mut app.contents);

        app.contents
            .buffers
            .insert(parent_id, Buffer::Directory(DirectoryBuffer::default()));
        app.contents.buffers.insert(
            current_id,
            Buffer::Directory(DirectoryBuffer {
                path: keep.clone(),
                ..Default::default()
            }),
        );
        app.contents.buffers.insert(
            preview_id,
            Buffer::Content(ContentBuffer {
                path: removed_file.clone(),
                ..Default::default()
            }),
        );

        let window = app.current_window_mut().expect("test requires current tab");
        *window = Window::Directory(
            ViewPort {
                buffer_id: parent_id,
                ..Default::default()
            },
            ViewPort {
                buffer_id: current_id,
                ..Default::default()
            },
            ViewPort {
                buffer_id: preview_id,
                ..Default::default()
            },
        );

        fs::remove_dir_all(&removed).expect("remove directory");

        let mut history = History::default();
        let mut marks = Marks::default();
        let mut qfix = QuickFix::default();
        let mut junk = JunkYard::default();

        let _ = remove(
            &mut history,
            &mut marks,
            &mut qfix,
            &mut junk,
            &Mode::Normal,
            &mut app,
            &removed,
        );

        let window = app.current_window().expect("test requires current tab");
        let (_, _, preview_id) = app::get_focused_directory_buffer_ids(window).unwrap();
        let preview_buffer = app.contents.buffers.get(&preview_id);
        assert!(preview_buffer.is_some());

        if let Some(path) = preview_buffer.and_then(|b| b.resolve_path()) {
            assert!(!path.starts_with(&removed));
        }

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn add_updates_directory_buffers_across_tabs() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");

        let added = base.join("added.txt");

        let mut app = App::default();
        app.tabs.clear();
        app.contents.buffers.clear();
        app.contents.latest_buffer_id = 0;

        insert_tab_with_directory(&mut app, 1, &base, Vec::new());
        insert_tab_with_directory(&mut app, 2, &base, Vec::new());
        app.current_tab_id = 1;

        let mut history = History::default();
        let marks = Marks::default();
        let qfix = QuickFix::default();
        let theme = Theme::default();

        let _ = add(
            &mut history,
            &marks,
            &qfix,
            &Mode::Navigation,
            &mut app,
            std::slice::from_ref(&added),
            &theme,
        )
        .expect("path add must succeed");

        for tab_id in [1, 2] {
            let window = app.tabs.get(&tab_id).expect("tab exists");
            let (_, current_id, _) = app::get_focused_directory_buffer_ids(window).unwrap();
            let buffer = match app.contents.buffers.get(&current_id) {
                Some(Buffer::Directory(buffer)) => buffer,
                _ => panic!("expected directory buffer"),
            };
            assert!(
                buffer
                    .buffer
                    .lines
                    .iter()
                    .any(|line| line.content.to_stripped_string() == "added.txt"),
                "expected added entry in tab {}",
                tab_id
            );
        }

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn remove_updates_directory_buffers_across_tabs() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");

        let removed = base.join("removed.txt");

        let mut app = App::default();
        app.tabs.clear();
        app.contents.buffers.clear();
        app.contents.latest_buffer_id = 0;

        insert_tab_with_directory(&mut app, 1, &base, vec!["removed.txt".to_string()]);
        insert_tab_with_directory(&mut app, 2, &base, vec!["removed.txt".to_string()]);
        app.current_tab_id = 1;

        let mut history = History::default();
        let mut marks = Marks::default();
        let mut qfix = QuickFix::default();
        let mut junk = JunkYard::default();

        let _ = remove(
            &mut history,
            &mut marks,
            &mut qfix,
            &mut junk,
            &Mode::Normal,
            &mut app,
            &removed,
        );

        for tab_id in [1, 2] {
            let window = app.tabs.get(&tab_id).expect("tab exists");
            let (_, current_id, _) = app::get_focused_directory_buffer_ids(window).unwrap();
            let buffer = match app.contents.buffers.get(&current_id) {
                Some(Buffer::Directory(buffer)) => buffer,
                _ => panic!("expected directory buffer"),
            };
            assert!(
                !buffer
                    .buffer
                    .lines
                    .iter()
                    .any(|line| line.content.to_stripped_string() == "removed.txt"),
                "expected removed entry in tab {}",
                tab_id
            );
        }

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn add_updates_shown_viewports_in_split() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");

        let added = base.join("added.txt");

        let mut app = App::default();
        app.tabs.clear();
        app.contents.buffers.clear();
        app.contents.latest_buffer_id = 0;

        insert_tab_with_directory(&mut app, 1, &base, Vec::new());
        let mut second = Window::create(
            app::get_empty_buffer(&mut app.contents),
            app::get_empty_buffer(&mut app.contents),
            app::get_empty_buffer(&mut app.contents),
        );

        if let Window::Directory(_, current, _) = &mut second {
            current.buffer_id = app::get_next_buffer_id(&mut app.contents);
            app.contents.buffers.insert(
                current.buffer_id,
                Buffer::Directory(DirectoryBuffer {
                    path: base.clone(),
                    ..Default::default()
                }),
            );
        }

        let split = Window::Horizontal {
            first: Box::new(app.tabs.remove(&1).expect("tab exists")),
            second: Box::new(second),
            focus: crate::model::SplitFocus::First,
        };
        app.tabs.insert(1, split);
        app.current_tab_id = 1;

        let mut history = History::default();
        let marks = Marks::default();
        let qfix = QuickFix::default();
        let theme = Theme::default();

        let _ = add(
            &mut history,
            &marks,
            &qfix,
            &Mode::Navigation,
            &mut app,
            std::slice::from_ref(&added),
            &theme,
        )
        .expect("path add must succeed");

        let window = app.current_window().expect("current window");
        let (first, second) = match window {
            Window::Horizontal { first, second, .. } => (first.as_ref(), second.as_ref()),
            _ => panic!("expected horizontal split"),
        };

        for target in [first, second] {
            let (_, current_id, _) = app::get_focused_directory_buffer_ids(target).unwrap();
            let buffer = match app.contents.buffers.get(&current_id) {
                Some(Buffer::Directory(buffer)) => buffer,
                _ => panic!("expected directory buffer"),
            };
            assert!(
                buffer
                    .buffer
                    .lines
                    .iter()
                    .any(|line| line.content.to_stripped_string() == "added.txt"),
                "expected added entry in shown viewport"
            );
        }

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn remove_updates_shown_viewports_in_split() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");

        let removed = base.join("removed.txt");

        let mut app = App::default();
        app.tabs.clear();
        app.contents.buffers.clear();
        app.contents.latest_buffer_id = 0;

        insert_tab_with_directory(&mut app, 1, &base, vec!["removed.txt".to_string()]);
        let mut second = Window::create(
            app::get_empty_buffer(&mut app.contents),
            app::get_empty_buffer(&mut app.contents),
            app::get_empty_buffer(&mut app.contents),
        );

        if let Window::Directory(_, current, _) = &mut second {
            current.buffer_id = app::get_next_buffer_id(&mut app.contents);
            app.contents.buffers.insert(
                current.buffer_id,
                Buffer::Directory(DirectoryBuffer {
                    path: base.clone(),
                    buffer: TextBuffer::from_lines(vec![BufferLine {
                        content: Ansi::new("removed.txt"),
                        ..Default::default()
                    }]),
                    ..Default::default()
                }),
            );
        }

        let split = Window::Horizontal {
            first: Box::new(app.tabs.remove(&1).expect("tab exists")),
            second: Box::new(second),
            focus: crate::model::SplitFocus::First,
        };
        app.tabs.insert(1, split);
        app.current_tab_id = 1;

        let mut history = History::default();
        let mut marks = Marks::default();
        let mut qfix = QuickFix::default();
        let mut junk = JunkYard::default();

        let _ = remove(
            &mut history,
            &mut marks,
            &mut qfix,
            &mut junk,
            &Mode::Normal,
            &mut app,
            &removed,
        );

        let window = app.current_window().expect("current window");
        let (first, second) = match window {
            Window::Horizontal { first, second, .. } => (first.as_ref(), second.as_ref()),
            _ => panic!("expected horizontal split"),
        };

        for target in [first, second] {
            let (_, current_id, _) = app::get_focused_directory_buffer_ids(target).unwrap();
            let buffer = match app.contents.buffers.get(&current_id) {
                Some(Buffer::Directory(buffer)) => buffer,
                _ => panic!("expected directory buffer"),
            };
            assert!(
                !buffer
                    .buffer
                    .lines
                    .iter()
                    .any(|line| line.content.to_stripped_string() == "removed.txt"),
                "expected removed entry in shown viewport"
            );
        }

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn remove_refreshes_preview_viewports_in_split_when_selected_entry_removed() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");

        let removed = base.join("removed.txt");
        let keep = base.join("keep.txt");
        fs::write(&removed, "content").expect("create removed file");
        fs::write(&keep, "content").expect("create keep file");

        let mut app = App::default();
        app.tabs.clear();
        app.contents.buffers.clear();
        app.contents.latest_buffer_id = 0;

        let _ = insert_shared_current_split(
            &mut app,
            &base,
            vec!["removed.txt".to_string(), "keep.txt".to_string()],
        );
        set_split_current_cursors(&mut app, 0, 0);

        let preview_first = app::get_next_buffer_id(&mut app.contents);
        let preview_second = app::get_next_buffer_id(&mut app.contents);
        app.contents.buffers.insert(
            preview_first,
            Buffer::Content(ContentBuffer {
                path: removed.clone(),
                buffer: TextBuffer::from_lines(vec![BufferLine {
                    content: Ansi::new("content"),
                    ..Default::default()
                }]),
            }),
        );
        app.contents.buffers.insert(
            preview_second,
            Buffer::Content(ContentBuffer {
                path: removed.clone(),
                buffer: TextBuffer::from_lines(vec![BufferLine {
                    content: Ansi::new("content"),
                    ..Default::default()
                }]),
            }),
        );

        let window = app.current_window_mut().expect("current window");
        match window {
            Window::Horizontal { first, second, .. } => {
                if let Window::Directory(_, _, preview) = first.as_mut() {
                    preview.buffer_id = preview_first;
                }
                if let Window::Directory(_, _, preview) = second.as_mut() {
                    preview.buffer_id = preview_second;
                }
            }
            _ => panic!("expected horizontal split"),
        }

        let mut history = History::default();
        let mut marks = Marks::default();
        let mut qfix = QuickFix::default();
        let mut junk = JunkYard::default();

        let _ = remove(
            &mut history,
            &mut marks,
            &mut qfix,
            &mut junk,
            &Mode::Normal,
            &mut app,
            &removed,
        );

        let window = app.current_window().expect("current window");
        let (first, second) = match window {
            Window::Horizontal { first, second, .. } => (first.as_ref(), second.as_ref()),
            _ => panic!("expected horizontal split"),
        };

        for target in [first, second] {
            let (_, _, preview_id) = app::get_focused_directory_buffer_ids(target).unwrap();
            let preview_path = app
                .contents
                .buffers
                .get(&preview_id)
                .and_then(|buffer| buffer.resolve_path())
                .map(|path| path.to_path_buf());
            assert!(
                preview_path.as_ref() != Some(&removed),
                "expected preview to refresh away from removed entry"
            );
        }

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn remove_refreshes_unfocused_preview_in_split() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");

        let removed = base.join("removed.txt");
        let keep = base.join("keep.txt");
        fs::write(&removed, "content").expect("create removed file");
        fs::write(&keep, "content").expect("create keep file");

        let mut app = App::default();
        app.tabs.clear();
        app.contents.buffers.clear();
        app.contents.latest_buffer_id = 0;

        let _ = insert_shared_current_split(
            &mut app,
            &base,
            vec!["removed.txt".to_string(), "keep.txt".to_string()],
        );
        set_split_current_cursors(&mut app, 0, 0);

        let preview_first = app::get_next_buffer_id(&mut app.contents);
        let preview_second = app::get_next_buffer_id(&mut app.contents);
        app.contents
            .buffers
            .insert(preview_first, Buffer::PathReference(removed.clone()));
        app.contents
            .buffers
            .insert(preview_second, Buffer::PathReference(removed.clone()));

        let window = app.current_window_mut().expect("current window");
        if let Window::Horizontal {
            first,
            second,
            focus,
        } = window
        {
            *focus = crate::model::SplitFocus::First;
            if let Window::Directory(_, _, preview) = first.as_mut() {
                preview.buffer_id = preview_first;
            }
            if let Window::Directory(_, _, preview) = second.as_mut() {
                preview.buffer_id = preview_second;
            }
        }

        let mut history = History::default();
        let mut marks = Marks::default();
        let mut qfix = QuickFix::default();
        let mut junk = JunkYard::default();

        let _ = remove(
            &mut history,
            &mut marks,
            &mut qfix,
            &mut junk,
            &Mode::Normal,
            &mut app,
            &removed,
        );

        let window = app.current_window().expect("current window");
        let (first, second) = match window {
            Window::Horizontal { first, second, .. } => (first.as_ref(), second.as_ref()),
            _ => panic!("expected horizontal split"),
        };

        for target in [first, second] {
            let (_, _, preview_id) = app::get_focused_directory_buffer_ids(target).unwrap();
            let preview_path = app
                .contents
                .buffers
                .get(&preview_id)
                .and_then(|buffer| buffer.resolve_path())
                .map(|path| path.to_path_buf());
            assert_eq!(preview_path, Some(keep.clone()));
        }

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn add_updates_all_shown_viewports_for_shared_buffer() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");

        let added = base.join("a.txt");

        let mut app = App::default();
        app.tabs.clear();
        app.contents.buffers.clear();
        app.contents.latest_buffer_id = 0;

        let shared_id = insert_shared_current_split(&mut app, &base, vec!["b.txt".to_string()]);
        set_split_current_cursors(&mut app, 0, 0);

        let mut history = History::default();
        let marks = Marks::default();
        let qfix = QuickFix::default();
        let theme = Theme::default();

        let _ = add(
            &mut history,
            &marks,
            &qfix,
            &Mode::Navigation,
            &mut app,
            std::slice::from_ref(&added),
            &theme,
        )
        .expect("path add must succeed");

        let shared_buffer = match app.contents.buffers.get(&shared_id) {
            Some(Buffer::Directory(buffer)) => buffer,
            _ => panic!("expected shared directory buffer"),
        };
        let expected_index = shared_buffer
            .buffer
            .lines
            .iter()
            .position(|line| line.content.to_stripped_string() == "b.txt")
            .expect("expected existing line");

        let (first_index, second_index) = split_current_indices(&app);
        assert_eq!(first_index, expected_index);
        assert_eq!(second_index, expected_index);

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn remove_updates_all_shown_viewports_for_shared_buffer() {
        let base = unique_temp_dir();
        fs::create_dir_all(&base).expect("create base dir");

        let removed = base.join("a.txt");

        let mut app = App::default();
        app.tabs.clear();
        app.contents.buffers.clear();
        app.contents.latest_buffer_id = 0;

        let _ = insert_shared_current_split(
            &mut app,
            &base,
            vec!["a.txt".to_string(), "b.txt".to_string()],
        );
        set_split_current_cursors(&mut app, 1, 1);

        let mut history = History::default();
        let mut marks = Marks::default();
        let mut qfix = QuickFix::default();
        let mut junk = JunkYard::default();

        let _ = remove(
            &mut history,
            &mut marks,
            &mut qfix,
            &mut junk,
            &Mode::Normal,
            &mut app,
            &removed,
        );

        let (first_index, second_index) = split_current_indices(&app);
        assert_eq!(first_index, 0);
        assert_eq!(second_index, 0);

        let _ = fs::remove_dir_all(&base);
    }
}
