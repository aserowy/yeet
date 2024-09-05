use std::{path::Path, process::Stdio, str};

use ratatui::layout::Rect;
use tokio::process::Command;

pub async fn load<'a>(path: &Path, rect: &Rect) -> Option<String> {
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
                // TODO: log error
            } else if output.stdout.is_empty() {
                // TODO: log error
            }

            let text = output.stdout;

            str::from_utf8(&text).ok().map(|s| s.to_string())
        }
        // TODO: log error
        Err(_) => None,
    }
}
