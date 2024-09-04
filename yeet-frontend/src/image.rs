use std::{path::Path, process::Stdio};

use ansi_to_tui::IntoText;
use ratatui::{layout::Rect, text::Line};
use tokio::process::Command;

pub async fn load<'a>(path: &Path, rect: &Rect) -> Option<Vec<Line<'a>>> {
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

            let text = output.stdout.to_text().unwrap();

            Some(text.lines)
            None
        }
        // TODO: log error
        Err(_) => None,
    }
}
