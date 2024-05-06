use crate::{
    action::Action,
    model::{mark::MARK_SIGN_ID, Model},
    task::Task,
};

use super::{
    selection::{get_current_selected_bufferline, get_current_selected_path},
    sign::{set_sign, unset_sign_for_path},
};

pub fn add_mark(model: &mut Model, char: char) -> Vec<Action> {
    let selected = get_current_selected_path(model);
    if let Some(selected) = selected {
        let removed = model.marks.entries.insert(char, selected);
        if let Some(removed) = removed {
            unset_sign_for_path(model, &removed, MARK_SIGN_ID);
        }

        if let Some(bl) = get_current_selected_bufferline(model) {
            set_sign(bl, MARK_SIGN_ID);
        }
    }
    Vec::new()
}

pub fn delete_mark(model: &mut Model, delete: &Vec<char>) -> Vec<Action> {
    let mut persisted = Vec::new();
    for mark in delete {
        let deleted = model.marks.entries.remove_entry(mark);
        if let Some((mark, path)) = deleted {
            unset_sign_for_path(model, path.as_path(), MARK_SIGN_ID);
            persisted.push(mark);
        }
    }

    if persisted.is_empty() {
        Vec::new()
    } else {
        vec![Action::Task(Task::DeleteMarks(persisted))]
    }
}
