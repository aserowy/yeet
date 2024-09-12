use std::{path::Path, process::Stdio, str};

use ratatui::layout::Rect;
use tokio::process::Command;

#[tracing::instrument]
pub async fn load<'a>(path: &Path, rect: &Rect) -> Option<String> {
    tracing::trace!("load image preview for path: {:?}", path);

    let result = Command::new("chafa")
        .args([
            "-f",
            "symbols",
            "--relative",
            "off",
            "--polite",
            "on",
            "--passthrough",
            "none",
            "--animate",
            "off",
            "--view-size",
        ])
        .arg(format!("{}x{}", rect.width, rect.height))
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .output()
        .await;

    match result {
        Ok(output) => {
            if !output.status.success() {
                tracing::error!("chafa failed: {:?}", output);
            } else if output.stdout.is_empty() {
                tracing::error!("chafa failed: image result is empty");
            }

            let text = output.stdout;

            str::from_utf8(&text).ok().map(|s| s.to_string())
        }
        Err(err) => {
            tracing::error!("chafa failed: {:?}", err);
            None
        }
    }
}
