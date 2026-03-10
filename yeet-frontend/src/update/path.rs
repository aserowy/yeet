use std::{
    path::{Path, PathBuf},
    slice,
};

use yeet_buffer::{message::BufferMessage, model::Mode};

use crate::{
    action::Action,
    error::AppError,
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
) -> Result<Vec<Action>, AppError> {
    let (window, contents) = app.current_window_and_contents_mut()?;
    let (current_vp, current_buffer) = app::get_focused_current_mut(window, contents)?;
    let previous_selection = match current_buffer {
        Buffer::Directory(buffer) => model::get_selected_path(buffer, &current_vp.cursor),
        _ => None,
    };

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
) -> Result<Vec<Action>, AppError> {
    if path.starts_with(junk.path.clone()) {
        remove_from_junkyard(junk, path);
    }

    history::remove_entry(history, path);

    let actions = update_directory_buffers_on_remove(history, mode, app, path)?;
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

fn update_directory_buffers_on_add(mode: &Mode, app: &mut App, path: &Path) {
    let (parent, name) = match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => (parent, name.to_string_lossy().to_string()),
        _ => return,
    };

    let (window, contents) = match app.current_window_and_contents_mut() {
        Ok(window) => window,
        Err(_) => return,
    };
    for (buffer_id, buffer) in contents.buffers.iter_mut() {
        let Buffer::Directory(dir) = buffer else {
            continue;
        };

        if dir.path != parent {
            continue;
        }

        let mut viewport = app::get_viewport_by_buffer_id_mut(window, *buffer_id);

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
                viewport.as_deref_mut(),
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
    }
}

fn update_directory_buffers_on_remove(
    history: &History,
    mode: &Mode,
    app: &mut App,
    path: &Path,
) -> Result<Vec<Action>, AppError> {
    let parent_name = match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => Some((parent, name.to_string_lossy().to_string())),
        _ => None,
    };

    if let Some((parent, name)) = parent_name {
        let (window, contents) = app.current_window_and_contents_mut()?;
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
        }
    }

    let window = app.current_window()?;
    let (current_id, preview_id) = match app::get_focused_directory_buffer_ids(window) {
        Some((_, current_id, preview_id)) => (current_id, preview_id),
        None => {
            return Err(AppError::InvalidState(
                "expected a directory window with focused current and preview buffers".to_string(),
            ))
        }
    };
    let current_path = match app.contents.buffers.get(&current_id) {
        Some(Buffer::Directory(buffer)) => buffer.resolve_path().map(|p| p.to_path_buf()),
        _ => {
            return Err(AppError::InvalidState(
                "expected current buffer to be a directory buffer".to_string(),
            ))
        }
    };
    let preview_path = app
        .contents
        .buffers
        .get(&preview_id)
        .and_then(|b| b.resolve_path())
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

        return Ok(actions);
    }

    if preview_path
        .as_ref()
        .map(|preview| preview.starts_with(path))
        .unwrap_or(false)
    {
        actions.extend(selection::refresh_preview_from_current_selection(
            app, history, None,
        )?);
    }

    Ok(actions)
}

fn remove_buffers_under_path(app: &mut App, path: &Path) {
    app.contents.buffers.retain(|_, buffer| {
        buffer
            .resolve_path()
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
    let buffer_id = app::get_empty_buffer(&mut app.contents);
    let (window, contents) = match app.current_window_and_contents_mut() {
        Ok(window) => window,
        Err(_) => return,
    };
    if let Some((parent, current, _)) = app::get_focused_directory_viewports_mut(window) {
        parent.buffer_id = buffer_id;
        current.buffer_id = buffer_id;
    }

    preview::set_buffer_id(contents, window, buffer_id);
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
        use crate::model::{DirectoryBuffer, Window};
        use yeet_buffer::model::{ansi::Ansi, viewport::ViewPort, BufferLine, Cursor, TextBuffer};

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

        let history = History::default();
        let marks = Marks::default();
        let qfix = QuickFix::default();

        // Simulate PathsAdded for the new folder (path without trailing slash,
        // as the filesystem watcher reports it).
        let actions = add(
            &history,
            &marks,
            &qfix,
            &Mode::Navigation,
            &mut app,
            std::slice::from_ref(&newfolder),
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
        use crate::model::{DirectoryBuffer, Window};
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
}
