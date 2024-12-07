use crate::{
    action::Action,
    model::{
        mark::{Marks, MARK_SIGN_ID},
        FileTreeBuffer,
    },
    task::Task,
};

use super::{
    selection::{get_current_selected_bufferline, get_current_selected_path},
    sign::{set, unset_sign_for_path},
};

pub fn add(marks: &mut Marks, buffer: &mut FileTreeBuffer, char: char) -> Vec<Action> {
    let selected = get_current_selected_path(buffer);
    if let Some(selected) = selected {
        let removed = marks.entries.insert(char, selected);
        if let Some(removed) = removed {
            // NOTE: all file tree buffer must get handled here
            unset_sign_for_path(buffer, &removed, MARK_SIGN_ID);
        }

        if let Some(bl) = get_current_selected_bufferline(buffer) {
            set(bl, MARK_SIGN_ID);
        }
    }
    Vec::new()
}

pub fn delete(marks: &mut Marks, buffer: &mut FileTreeBuffer, delete: &Vec<char>) -> Vec<Action> {
    let mut persisted = Vec::new();
    for mark in delete {
        let deleted = marks.entries.remove_entry(mark);
        if let Some((mark, path)) = deleted {
            // NOTE: all file tree buffer must get handled here
            unset_sign_for_path(buffer, path.as_path(), MARK_SIGN_ID);
            persisted.push(mark);
        }
    }

    if persisted.is_empty() {
        Vec::new()
    } else {
        vec![Action::Task(Task::DeleteMarks(persisted))]
    }
}
