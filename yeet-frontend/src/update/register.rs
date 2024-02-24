use std::path::PathBuf;

use crate::{action::Action, model::Model, task::Task};

use super::model::current;

pub fn add(model: &mut Model, paths: &Vec<PathBuf>) -> Option<Vec<Action>> {
    let mut actions = vec![Action::SkipRender];
    for path in paths {
        if path.starts_with(&model.register.path) {
            if let Some(old_entry) = model.register.add_or_update(path) {
                actions.push(Action::Task(Task::DeleteRegisterEntry(old_entry)));
            }
        }
    }

    Some(actions)
}

pub fn paste(model: &mut Model, register: &str) -> Option<Vec<Action>> {
    if let Some(entry) = model.register.get(register) {
        Some(vec![Action::Task(Task::RestorePath(
            entry,
            model.current.path.clone(),
        ))])
    } else {
        None
    }
}

pub fn yank(model: &mut Model) -> Option<Vec<Action>> {
    if let Some(selected) = current::selection(model) {
        let mut tasks = Vec::new();

        let (entry, old_entry) = model.register.yank(&selected);
        tasks.push(Action::Task(Task::YankPath(entry)));
        if let Some(old_entry) = old_entry {
            tasks.push(Action::Task(Task::DeleteRegisterEntry(old_entry)));
        }

        Some(tasks)
    } else {
        None
    }
}
