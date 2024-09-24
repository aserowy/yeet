use std::{path::Path, process::Stdio, str};

use ratatui::layout::Rect;
use tokio::process::Command;

use crate::event::Preview;

#[tracing::instrument]
pub async fn load<'a>(path: &Path, rect: &Rect) -> Preview {
    load_with_chafa(path, rect).await
}

async fn load_with_chafa(path: &Path, rect: &Rect) -> Preview {
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
            format!("{}x{}", rect.width, rect.height).as_str(),
        ])
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
                Preview::None(path.to_path_buf())
            } else if output.stdout.is_empty() {
                tracing::warn!("chafa failed: image result is empty");
                Preview::None(path.to_path_buf())
            } else {
                let content = str::from_utf8(&output.stdout)
                    .map_or(vec![], |s| s.lines().map(|l| l.to_string()).collect());

                Preview::Content(path.to_path_buf(), content)
            }
        }
        Err(err) => {
            tracing::error!("chafa failed: {:?}", err);
            Preview::None(path.to_path_buf())
        }
    }
}
