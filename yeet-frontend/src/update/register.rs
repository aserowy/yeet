use std::path::PathBuf;

use crate::{action::Action, model::Model, task::Task};

pub fn add(model: &mut Model, paths: &Vec<PathBuf>) -> Vec<Action> {
    let mut actions = Vec::new();
    for path in paths {
        if path.starts_with(&model.junk.path) {
            if let Some(obsolete) = model.junk.add_or_update(path) {
                for entry in obsolete.entries {
                    actions.push(Action::Task(Task::DeleteJunkYardEntry(entry)));
                }
            }
        }
    }

    actions
}

pub fn paste(model: &mut Model, register: &char) -> Vec<Action> {
    if let Some(transaction) = model.junk.get(register) {
        let mut actions = Vec::new();
        for entry in transaction.entries {
            actions.push(Action::Task(Task::RestorePath(
                entry,
                model.file_buffer.current.path.clone(),
            )));
        }
        actions
    } else {
        Vec::new()
    }
}

pub fn yank(model: &mut Model, repeat: &usize) -> Vec<Action> {
    let current_buffer = &model.file_buffer.current.buffer;
    if current_buffer.lines.is_empty() {
        Vec::new()
    } else if let Some(cursor) = &current_buffer.cursor {
        let mut paths = Vec::new();
        for rpt in 0..*repeat {
            let line_index = cursor.vertical_index + rpt;
            if let Some(line) = current_buffer.lines.get(line_index) {
                let target = model.file_buffer.current.path.join(&line.content);
                paths.push(target);
            }
        }

        let mut actions = Vec::new();
        let (transaction, obsolete) = model.junk.yank(paths);
        for entry in transaction.entries {
            actions.push(Action::Task(Task::YankPath(entry)));
        }

        if let Some(obsolete) = obsolete {
            for entry in obsolete.entries {
                actions.push(Action::Task(Task::DeleteJunkYardEntry(entry)));
            }
        }

        actions
    } else {
        Vec::new()
    }
}
