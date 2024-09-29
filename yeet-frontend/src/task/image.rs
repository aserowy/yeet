use std::{path::Path, process::Stdio, str};

use image::ImageReader;
use ratatui::layout::Rect;
use ratatui_image::{picker::Picker, Resize};
use tokio::process::Command;

use crate::{error::AppError, event::Preview};

#[tracing::instrument]
pub async fn load<'a>(picker: &mut Option<Picker>, path: &Path, rect: &Rect) -> Preview {
    let picker = match picker {
        Some(pckr) => pckr,
        None => return load_with_chafa(path, rect).await,
    };

    match load_with_ratatui_image(picker, path, rect).await {
        Ok(preview) => preview,
        Err(err) => {
            tracing::error!("image preview failed: {:?}", err);
            load_with_chafa(path, rect).await
        }
    }
}

async fn load_with_ratatui_image(
    picker: &mut Picker,
    path: &Path,
    rect: &Rect,
) -> Result<Preview, AppError> {
    tracing::debug!("load image preview for path with ratatui image: {:?}", path);

    let image = ImageReader::open(path)?.decode()?;
    picker.guess_protocol();

    match picker.new_protocol(image, rect.clone(), Resize::Fit(None)) {
        Ok(prtcl) => Ok(Preview::Image(path.to_path_buf(), prtcl)),
        Err(err) => {
            tracing::error!("Generation of preview image protocol failed: {:?}", err);
            Err(AppError::PreviewProtocolGenerationFailed)
        }
    }
}

async fn load_with_chafa(path: &Path, rect: &Rect) -> Preview {
    tracing::debug!("load image preview for path with chafa: {:?}", path);

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
