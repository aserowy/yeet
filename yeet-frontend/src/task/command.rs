use std::{
    path::{Path, PathBuf},
    process::Stdio,
    str,
};

use tokio::process::Command;

use crate::error::AppError;

pub async fn fd(base_path: &Path, params: String) -> Result<Vec<PathBuf>, AppError> {
    tracing::debug!("executing fd at {:?} with {:?} params", base_path, params);

    let params = params.split(" ");
    let result = Command::new("fd")
        .args(["--color", "never", "--absolute-path", "--base-directory"])
        .arg(base_path)
        .args(params)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await;

    match result {
        Ok(output) => {
            if !output.status.success() {
                let message = format!("fd failed: {:?}", output);
                tracing::error!(message);
                Err(AppError::ExecutionFailed(message))
            } else if output.stdout.is_empty() {
                let message = format!("fd failed: result is empty");
                tracing::error!(message);
                Err(AppError::ExecutionFailed(message))
            } else {
                let result = str::from_utf8(&output.stdout).map_or(vec![], |s| {
                    s.lines()
                        .map(|l| l.to_string())
                        .filter_map(|s| {
                            let path = PathBuf::from(s);
                            if path.exists() {
                                Some(path)
                            } else {
                                None
                            }
                        })
                        .collect()
                });
                Ok(result)
            }
        }
        Err(err) => {
            let message = format!("fd failed: {:?}", err);
            tracing::error!(message);
            Err(AppError::ExecutionFailed(message))
        }
    }
}
