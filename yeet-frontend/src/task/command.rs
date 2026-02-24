use std::{
    io::Error,
    path::{Path, PathBuf},
    process::{Output, Stdio},
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
                let message = "fd failed: result is empty".to_string();
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

pub async fn rg(base_path: &Path, params: String) -> Result<Vec<PathBuf>, AppError> {
    tracing::debug!("executing rg at {:?} with {:?} params", base_path, params);

    if base_path.is_relative() {
        let message = format!("rg failed: base path is not absolute: {:?}", base_path);
        tracing::error!(message);
        return Err(AppError::ExecutionFailed(message));
    }

    let params = params.split(" ");
    let result = Command::new("rg")
        .args(["--color", "never", "--files-with-matches"])
        .args(params)
        .arg(base_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await;

    resolve_output_result("rg", result)
}

pub async fn zoxide(params: String) -> Result<PathBuf, AppError> {
    tracing::debug!("executing zoxide with {:?} params", params);

    let result = Command::new("zoxide")
        .arg("query")
        .arg(params)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await;

    let result = resolve_output_result("zoxide", result)?;
    if let Some(target) = result.into_iter().next() {
        Ok(target)
    } else {
        Err(AppError::ExecutionFailed(
            "zoxide failed: no valid path found".to_string(),
        ))
    }
}

fn resolve_output_result(
    prefix: &str,
    result: Result<Output, Error>,
) -> Result<Vec<PathBuf>, AppError> {
    match result {
        Ok(output) => {
            if !output.status.success() {
                let message = format!("{:?} failed: {:?}", prefix, output);
                tracing::error!(message);
                Err(AppError::ExecutionFailed(message))
            } else if output.stdout.is_empty() {
                let message = format!("{:?} returned no valid paths", prefix);
                tracing::info!(message);
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
            let message = format!("{:?} failed: {:?}", prefix, err);
            tracing::error!(message);
            Err(AppError::ExecutionFailed(message))
        }
    }
}
