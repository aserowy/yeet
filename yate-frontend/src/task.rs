use tokio::task::JoinSet;

use crate::model::history;

#[derive(Clone, Debug, PartialEq)]
pub enum Task {
    OptimizeHistory,
}

#[derive(Default)]
pub struct TaskManager {
    tasks: JoinSet<()>,
}

impl TaskManager {
    pub fn run(&mut self, task: Task) {
        match task {
            Task::OptimizeHistory => self.tasks.spawn(async move {
                history::cache::optimize();
            }),
        };
    }

    pub async fn finishing(&mut self) {
        while let Some(task) = self.tasks.join_next().await {
            match task {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    }
}
