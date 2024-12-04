use std::cmp::Ordering;

use tokio_util::sync::CancellationToken;

use crate::{
    action::Action,
    model::{CurrentTask, Model},
};

pub fn add(model: &mut Model, identifier: String, cancellation: CancellationToken) -> Vec<Action> {
    let id = next_id(model);

    if let Some(replaced_task) = model.current_tasks.insert(
        identifier.clone(),
        CurrentTask {
            token: cancellation,
            id,
            external_id: identifier,
        },
    ) {
        replaced_task.token.cancel();
    }
    Vec::new()
}

fn next_id(model: &mut Model) -> u16 {
    let mut next_id = if model.latest_task_id >= 9999 {
        1
    } else {
        model.latest_task_id + 1
    };

    let mut running_ids: Vec<u16> = model.current_tasks.values().map(|task| task.id).collect();
    running_ids.sort();

    for id in running_ids {
        match next_id.cmp(&id) {
            Ordering::Equal => next_id += 1,
            Ordering::Greater => break,
            Ordering::Less => {}
        }
    }

    model.latest_task_id = next_id;

    next_id
}

pub fn remove(model: &mut Model, identifier: String) -> Vec<Action> {
    if let Some(task) = model.current_tasks.remove(&identifier) {
        task.token.cancel();
    }
    Vec::new()
}
