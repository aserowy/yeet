use crate::{action::Action, model::Model};

pub fn delete(model: &mut Model, id: &str) -> Vec<Action> {
    if let Some(task) = model.current_tasks.get_mut(id) {
    task.cancel();
    }
    Vec::new()
}
