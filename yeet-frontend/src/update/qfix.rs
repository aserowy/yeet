use std::path::PathBuf;

use crate::{
    action::Action,
    model::{
        qfix::{QuickFix, QFIX_SIGN_ID},
        FileTreeBuffer,
    },
};

use super::{
    selection::{get_current_selected_bufferline, get_current_selected_path},
    sign,
};

pub fn toggle_selected_to_qfix( qfix: &mut QuickFix,buffer: &mut FileTreeBuffer) -> Vec<Action> {
    let selected = get_current_selected_path(buffer);
    if let Some(selected) = selected {
        if qfix.entries.contains(&selected) {
            qfix.entries.retain(|p| p != &selected);
            if let Some(bl) = get_current_selected_bufferline(buffer) {
                sign::unset(bl, QFIX_SIGN_ID);
            }
        } else {
            qfix.entries.push(selected);
            if let Some(bl) = get_current_selected_bufferline(buffer) {
                sign::set(bl, QFIX_SIGN_ID);
            }
        }
    }
    Vec::new()
}

pub fn add(qfix: &mut QuickFix, buffer: &mut FileTreeBuffer, paths: Vec<PathBuf>) -> Vec<Action> {
    for path in paths {
        if !qfix.entries.contains(&path) {
            sign::set_sign_for_path(buffer, path.as_path(), QFIX_SIGN_ID);
            qfix.entries.push(path);
        };
    }
    Vec::new()
}
