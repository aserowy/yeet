use std::{fs, io, path::PathBuf};

use thiserror::Error;
use tokio::task::JoinSet;

use crate::model::history;

#[derive(Clone, Debug, PartialEq)]
pub enum Task {
    DeleteFile(PathBuf),
    OptimizeHistory,
}

#[derive(Default)]
pub struct TaskManager {
    tasks: JoinSet<Result<(), TaskError>>,
}

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Failed to delete file")]
    FileOperationFailed(#[from] io::Error),
    #[error("Target is invalid")]
    InvalidTarget,
    #[error("Failed to optimize history")]
    _OptimizeHistory,
}

impl TaskManager {
    // TODO: if error occurs, enable handling in model with RenderAction + sender
    pub fn run(&mut self, task: Task) {
        match task {
            Task::DeleteFile(path) => self.tasks.spawn(async move {
                if !path.exists() {
                    return Err(TaskError::InvalidTarget);
                }

                let result = if path.is_file() {
                    fs::remove_file(&path)
                } else if path.is_dir() {
                    fs::remove_dir_all(&path)
                } else {
                    Err(io::Error::new(io::ErrorKind::NotFound, "Target is invalid"))
                };

                match result {
                    Ok(_) => Ok(()),
                    Err(error) => Err(TaskError::FileOperationFailed(error)),
                }
            }),
            Task::OptimizeHistory => self.tasks.spawn(async move {
                // TODO: add error handling
                history::cache::optimize();

                Ok(())
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
