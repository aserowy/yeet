use std::{
    path::{Path, PathBuf},
    slice,
};

use yeet_buffer::{message::BufferMessage, model::Mode};

use crate::{
    action::Action,
    model::{
        self,
        history::History,
        junkyard::JunkYard,
        mark::{Marks, MARK_SIGN_ID},
        qfix::{QuickFix, QFIX_SIGN_ID},
        App, Buffer,
    },
    update::{app, preview, selection},
};

use super::{enumeration, history, junkyard::remove_from_junkyard, navigate, sign};

#[tracing::instrument(skip(app))]
pub fn add(
    history: &History,
    marks: &Marks,
    qfix: &QuickFix,
    mode: &Mode,
    app: &mut App,
    paths: &[PathBuf],
) -> Vec<Action> {
    let (_, current_id, _) = app::directory_buffer_ids(app);
    let current_vp = app::get_viewport_by_buffer_id(app, current_id);
    let current_cursor = current_vp.map(|vp| vp.cursor.clone());
    let previous_selection = app
        .buffers
        .get(&current_id)
        .and_then(|buffer| match buffer {
            Buffer::Directory(buffer) => current_cursor
                .as_ref()
                .and_then(|cursor| model::get_selected_path(buffer, cursor)),
            _ => None,
        });

    for path in paths {
        update_directory_buffers_on_add(mode, app, path);
    }

    let marked_paths: Vec<_> = paths
        .iter()
        .filter(|path| marks.entries.values().any(|marked| marked == *path))
        .cloned()
        .collect();
    if !marked_paths.is_empty() {
        sign::set_sign_for_paths(
            app.buffers.values_mut().collect(),
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
        sign::set_sign_for_paths(app.buffers.values_mut().collect(), qfix_paths, QFIX_SIGN_ID);
    }

    selection::refresh_preview_from_current_selection(app, history, previous_selection)
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
) -> Vec<Action> {
    if path.starts_with(junk.path.clone()) {
        remove_from_junkyard(junk, path);
    }

    history::remove_entry(history, path);

    let actions = update_directory_buffers_on_remove(history, mode, app, path);

    let removed_marks = remove_marks_for_path(marks, path);
    if !removed_marks.is_empty() {
        sign::unset_sign_for_paths(
            app.buffers.values_mut().collect(),
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
        app.buffers.values_mut().collect(),
        removed_qfix,
        QFIX_SIGN_ID,
    );

    actions
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

fn update_directory_buffers_on_add(mode: &Mode, app: &mut App, path: &Path) {
    let (parent, name) = match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => (parent, name.to_string_lossy().to_string()),
        _ => return,
    };

    for buffer in app.buffers.values_mut() {
        let Buffer::Directory(dir) = buffer else {
            continue;
        };

        if dir.path != parent {
            continue;
        }

        if dir
            .buffer
            .lines
            .iter()
            .any(|line| line.content.to_stripped_string() == name)
        {
            yeet_buffer::update(
                None,
                mode,
                &mut dir.buffer,
                std::slice::from_ref(&BufferMessage::SortContent(super::SORT)),
            );

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

        let bufferline = enumeration::from_enumeration(&name, &kind);
        if let Some(index) = added_existing_directory {
            if let Some(line) = dir.buffer.lines.get_mut(index) {
                *line = bufferline;
            }

            yeet_buffer::update(
                None,
                mode,
                &mut dir.buffer,
                std::slice::from_ref(&BufferMessage::SortContent(super::SORT)),
            );
        } else {
            yeet_buffer::update(
                None,
                mode,
                &mut dir.buffer,
                std::slice::from_ref(&BufferMessage::AddLine(bufferline, super::SORT)),
            );
        }
    }
}

fn update_directory_buffers_on_remove(
    history: &History,
    mode: &Mode,
    app: &mut App,
    path: &Path,
) -> Vec<Action> {
    let parent_name = match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => Some((parent, name.to_string_lossy().to_string())),
        _ => None,
    };

    if let Some((parent, name)) = parent_name {
        for buffer in app.buffers.values_mut() {
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

            indices.sort_unstable_by(|a, b| b.cmp(a));
            for index in indices {
                yeet_buffer::update(
                    None,
                    mode,
                    &mut dir.buffer,
                    slice::from_ref(&BufferMessage::RemoveLine(index)),
                );
            }
        }
    }

    let (_, current_id, preview_id) = app::directory_buffer_ids(app);
    let current_path = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(buffer)) => buffer.resolve_path().map(|p| p.to_path_buf()),
        _ => None,
    };
    let preview_path = app
        .buffers
        .get(&preview_id)
        .and_then(buffer_path)
        .map(|p| p.to_path_buf());

    let current_removed = current_path
        .as_ref()
        .map(|current| current.starts_with(path))
        .unwrap_or(false);

    remove_buffers_under_path(app, path);

    let mut actions = Vec::new();
    if current_removed {
        if let Some(existing) = find_existing_ancestor(path) {
            actions.extend(navigate::path(app, history, &existing));
        } else {
            reset_directory_viewports_to_empty(app);
        }

        return actions;
    }

    if preview_path
        .as_ref()
        .map(|preview| preview.starts_with(path))
        .unwrap_or(false)
    {
        actions.extend(selection::refresh_preview_from_current_selection(
            app, history, None,
        ));
    }

    actions
}

fn buffer_path(buffer: &Buffer) -> Option<&Path> {
    match buffer {
        Buffer::Directory(buffer) => buffer.resolve_path(),
        Buffer::Content(buffer) => buffer.resolve_path(),
        Buffer::Image(buffer) => buffer.resolve_path(),
        Buffer::PathReference(path) => Some(path.as_path()),
        Buffer::Empty => None,
    }
}

fn remove_buffers_under_path(app: &mut App, path: &Path) {
    app.buffers.retain(|_, buffer| {
        buffer_path(buffer)
            .map(|buffer_path| !buffer_path.starts_with(path))
            .unwrap_or(true)
    });
}

fn find_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut candidate = Some(path);
    while let Some(path) = candidate {
        if path.exists() {
            return Some(path.to_path_buf());
        }

        candidate = path.parent();
    }

    None
}

fn reset_directory_viewports_to_empty(app: &mut App) {
    let buffer_id = app::create_empty_buffer(app);
    let (parent, current, _) = app::directory_viewports_mut(app);
    parent.buffer_id = buffer_id;
    current.buffer_id = buffer_id;

    preview::set_buffer_id(app, buffer_id);
}

#[cfg(test)]
mod test {
    use std::{fs, time::SystemTime};

    use yeet_buffer::model::Mode;

    use crate::model::{
        history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, App, ContentBuffer,
    };

    use super::*;

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("yeet-path-test-{}", nanos))
    }

    #[test]
    fn removal_relocates_current_to_existing_ancestor_and_prunes_buffers() {
        let base = unique_temp_dir();
        let removed = base.join("removed");
        let nested = removed.join("inner");
        fs::create_dir_all(&nested).expect("create directories");

        let file_path = removed.join("file.txt");
        fs::write(&file_path, "content").expect("create file");

        let mut app = App::default();
        let (_, current_id, preview_id) = app::directory_buffer_ids(&app);

        if let Some(Buffer::Directory(buffer)) = app.buffers.get_mut(&current_id) {
            buffer.path = nested.clone();
        } else {
            panic!("expected current directory buffer");
        }

        app.buffers.insert(
            preview_id,
            Buffer::Content(ContentBuffer {
                path: file_path.clone(),
                ..Default::default()
            }),
        );

        let extra_id = app::get_next_buffer_id(&mut app);
        app.buffers.insert(
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

        remove(
            &mut history,
            &mut marks,
            &mut qfix,
            &mut junk,
            &Mode::Normal,
            &mut app,
            &removed,
        );

        let (_, current_id, _) = app::directory_buffer_ids(&app);
        let current_path = match app.buffers.get(&current_id) {
            Some(Buffer::Directory(buffer)) => buffer.path.clone(),
            Some(Buffer::PathReference(path)) => path.clone(),
            _ => PathBuf::new(),
        };

        assert_eq!(current_path, base);
        assert!(app
            .buffers
            .values()
            .filter_map(buffer_path)
            .all(|path| !path.starts_with(&removed)));

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn preview_is_reset_when_removed_from_subtree() {
        let base = unique_temp_dir();
        let keep = base.join("keep");
        let removed = base.join("removed");
        fs::create_dir_all(&keep).expect("create keep");
        fs::create_dir_all(&removed).expect("create removed");

        let removed_file = removed.join("gone.txt");
        fs::write(&removed_file, "content").expect("create file");

        let mut app = App::default();
        let (_, current_id, preview_id) = app::directory_buffer_ids(&app);

        if let Some(Buffer::Directory(buffer)) = app.buffers.get_mut(&current_id) {
            buffer.path = keep.clone();
        } else {
            panic!("expected current directory buffer");
        }

        app.buffers.insert(
            preview_id,
            Buffer::Content(ContentBuffer {
                path: removed_file.clone(),
                ..Default::default()
            }),
        );

        fs::remove_dir_all(&removed).expect("remove directory");

        let mut history = History::default();
        let mut marks = Marks::default();
        let mut qfix = QuickFix::default();
        let mut junk = JunkYard::default();

        remove(
            &mut history,
            &mut marks,
            &mut qfix,
            &mut junk,
            &Mode::Normal,
            &mut app,
            &removed,
        );

        let (_, _, preview_id) = app::directory_buffer_ids(&app);
        let preview_buffer = app.buffers.get(&preview_id);
        assert!(preview_buffer.is_some());

        if let Some(path) = preview_buffer.and_then(buffer_path) {
            assert!(!path.starts_with(&removed));
        }

        let _ = fs::remove_dir_all(&base);
    }
}
