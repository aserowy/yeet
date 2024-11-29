use crate::{action::Action, model::Model};

pub fn delete(model: &mut Model, id: u16) -> Vec<Action> {
    if let Some((_, task)) = model.current_tasks.iter().find(|(_, task)| task.id == id) {
        task.token.cancel();
    }
    Vec::new()
}
