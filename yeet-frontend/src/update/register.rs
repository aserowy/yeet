use std::path::PathBuf;

use crate::{action::Action, model::Model, task::Task};

use super::model::current;

pub fn add(model: &mut Model, paths: &Vec<PathBuf>) -> Option<Vec<Action>> {
    let mut actions = vec![Action::SkipRender];
    for path in paths {
        if path.starts_with(&model.register.path) {
            if let Some(obsolete) = model.register.add_or_update(path) {
                for entry in obsolete.entries {
                    actions.push(Action::Task(Task::DeleteRegisterEntry(entry)));
                }
            }
        }
    }

    Some(actions)
}

pub fn paste(model: &mut Model, register: &str) -> Option<Vec<Action>> {
    if let Some(transaction) = model.register.get(register) {
        let mut actions = Vec::new();
        for entry in transaction.entries {
            actions.push(Action::Task(Task::RestorePath(
                entry,
                model.current.path.clone(),
            )));
        }
        Some(actions)
    } else {
        None
    }
}

pub fn yank(model: &mut Model) -> Option<Vec<Action>> {
    if let Some(selected) = current::selection(model) {
        let mut tasks = Vec::new();

        // TODO: multiple yanks into one transaction!
        let (transaction, obsolete) = model.register.yank(vec![selected]);
        for entry in transaction.entries {
            tasks.push(Action::Task(Task::YankPath(entry)));
        }

        if let Some(obsolete) = obsolete {
            for entry in obsolete.entries {
                tasks.push(Action::Task(Task::DeleteRegisterEntry(entry)));
            }
        }

        Some(tasks)
    } else {
        None
    }
}
