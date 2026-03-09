use std::path::PathBuf;

use crate::{
    action::Action,
    model::{
        self,
        qfix::{QuickFix, QFIX_SIGN_ID},
        App, Buffer,
    },
};

use super::{app, sign};

pub fn toggle(app: &mut App, qfix: &mut QuickFix) -> Vec<Action> {
    let (window, contents) = match app.current_window_and_contents_mut() {
        Ok(window) => window,
        Err(_) => return Vec::new(),
    };
    let (vp, buffer) = app::get_focused_current_mut(window, contents);
    let (vp_cursor, buffer) = match buffer {
        Buffer::Directory(it) => (vp.cursor.clone(), it),
        Buffer::Image(_) => return Vec::new(),
        Buffer::Content(_) => return Vec::new(),
        Buffer::PathReference(_) => return Vec::new(),
        Buffer::Tasks(_) => return Vec::new(),
        Buffer::Empty => return Vec::new(),
    };

    let selected = model::get_selected_path(buffer, &vp_cursor);
    if let Some(selected) = selected {
        if qfix.entries.contains(&selected) {
            qfix.entries.retain(|p| p != &selected);

            sign::unset_sign_for_paths(
                app.contents.buffers.values_mut().collect(),
                vec![selected.clone()],
                QFIX_SIGN_ID,
            );
        } else {
            qfix.entries.push(selected.clone());

            sign::set_sign_for_paths(
                app.contents.buffers.values_mut().collect(),
                vec![selected],
                QFIX_SIGN_ID,
            );
        }
    }

    Vec::new()
}

pub fn add(qfix: &mut QuickFix, buffers: Vec<&mut Buffer>, paths: Vec<PathBuf>) -> Vec<Action> {
    let mut added_paths = Vec::new();
    for path in paths {
        if !qfix.entries.contains(&path) {
            added_paths.push(path.clone());
            qfix.entries.push(path);
        };
    }

    sign::set_sign_for_paths(buffers, added_paths, QFIX_SIGN_ID);

    Vec::new()
}
