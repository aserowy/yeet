use std::{
    path::{Path, PathBuf},
    slice,
};

use yeet_buffer::{message::BufferMessage, model::Mode};

use crate::{
    action::Action,
    model::{
        history::History,
        junkyard::JunkYard,
        mark::{Marks, MARK_SIGN_ID},
        qfix::{QuickFix, QFIX_SIGN_ID},
        App, Buffer,
    },
    update::{app, selection},
};

use super::{enumeration, history, junkyard::remove_from_junkyard, sign};

#[tracing::instrument(skip(app))]
pub fn add(
    history: &History,
    marks: &Marks,
    qfix: &QuickFix,
    mode: &Mode,
    app: &mut App,
    paths: &[PathBuf],
) {
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

    let removed_qfix = remove_qfix_for_path(qfix, path);
    if !removed_qfix.is_empty() {
        sign::unset_sign_for_paths(
            app.buffers.values_mut().collect(),
            removed_qfix,
            QFIX_SIGN_ID,
        );
    }

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

fn remove_qfix_for_path(qfix: &mut QuickFix, path: &Path) -> Vec<PathBuf> {
    let removed_paths: Vec<_> = qfix
        .entries
        .iter()
        .filter(|entry| entry.starts_with(path))
        .cloned()
        .collect();
    if !removed_paths.is_empty() {
        qfix.entries.retain(|entry| !entry.starts_with(path));
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
            continue;
        }

        let kind = if path.is_dir() {
            crate::event::ContentKind::Directory
        } else {
            crate::event::ContentKind::File
        };

        yeet_buffer::update(
            None,
            mode,
            &mut dir.buffer,
            std::slice::from_ref(&BufferMessage::AddLine(
                enumeration::from_enumeration(&name, &kind),
                super::SORT,
            )),
        );
    }
}

fn update_directory_buffers_on_remove(
    history: &History,
    mode: &Mode,
    app: &mut App,
    path: &Path,
) -> Vec<Action> {
    let (parent, name) = match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => (parent, name.to_string_lossy().to_string()),
        _ => return Vec::new(),
    };

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

    let (_, current_id, preview_id) = app::directory_buffer_ids(app);
    let preview_buffer = match app.buffers.get(&preview_id) {
        Some(Buffer::Directory(buffer)) => buffer,
        _ => return Vec::new(),
    };

    if !preview_buffer.path.eq(path) {
        return Vec::new();
    }

    let current_buffer = match app.buffers.get(&current_id) {
        Some(Buffer::Directory(buffer)) => buffer,
        _ => return Vec::new(),
    };

    let mut actions = Vec::new();
    if let Some(selected_path) =
        selection::get_current_selected_path(current_buffer, Some(&current_buffer.buffer.cursor))
    {
        let selection =
            history::get_selection_from_history(history, &selected_path).map(|s| s.to_owned());

        actions.push(Action::Load(selected_path.clone(), selection));

        let preview_id = app::get_or_create_directory_buffer_with_id(app, &selected_path);
        if let Some(Buffer::Directory(_)) = app.buffers.get(&preview_id) {
            let (_, _, preview) = app::directory_viewports_mut(app);
            preview.buffer_id = preview_id;
        }
    }

    actions
}
