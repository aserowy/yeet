use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use tokio::task::JoinSet;

use crate::{error::AppError, model::history};

#[derive(Clone, Debug, PartialEq)]
pub enum Task {
    AddPath(PathBuf),
    DeletePath(PathBuf),
    OptimizeHistory,
}

#[derive(Default)]
pub struct TaskManager {
    tasks: JoinSet<Result<(), AppError>>,
}

impl TaskManager {
    // TODO: if error occurs, enable handling in model with RenderAction + sender
    pub fn run(&mut self, task: Task) {
        match task {
            Task::AddPath(path) => self.tasks.spawn(async move {
                if path.exists() {
                    return Err(AppError::InvalidTargetPath);
                }

                let history_dictionary = match Path::new(&path).parent() {
                    Some(path) => path,
                    None => return Err(AppError::InvalidTargetPath),
                };

                fs::create_dir_all(history_dictionary)?;
                fs::write(path, "")?;

                Ok(())
            }),
            Task::DeletePath(path) => self.tasks.spawn(async move {
                if !path.exists() {
                    return Err(AppError::InvalidTargetPath);
                }

                if path.is_file() {
                    fs::remove_file(&path)?;
                } else if path.is_dir() {
                    fs::remove_dir_all(&path)?;
                };

                Ok(())
            }),
            Task::OptimizeHistory => self.tasks.spawn(async move {
                history::cache::optimize()?;

                Ok(())
            }),
        };
    }

    pub async fn finishing(&mut self) -> Result<(), AppError> {
        let mut errors = Vec::new();
        while let Some(task) = self.tasks.join_next().await {
            match task {
                Ok(Ok(())) => (),
                Ok(Err(error)) => errors.push(error),
                Err(_) => (), // TODO: log error
            };
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::Aggregate(errors))
        }
    }
}
