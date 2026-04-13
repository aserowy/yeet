use std::{path::Path, process::Stdio, str};

use image::ImageReader;
use ratatui::layout::Rect;
use ratatui_image::{picker::Picker, Resize};
use tokio::process::Command;

use crate::{error::AppError, event::Preview};

#[tracing::instrument]
pub async fn load(picker: &mut Option<Picker>, path: &Path, rect: &Rect) -> Preview {
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

    match picker.new_protocol(image, *rect, Resize::Fit(None)) {
        Ok(protocol) => Ok(Preview::Image(path.to_path_buf(), protocol)),
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
                let content = str::from_utf8(&output.stdout).map_or(vec![], |s| {
                    s.lines()
                        .map(strip_non_sgr_escape_sequences)
                        .collect()
                });

                Preview::Content(path.to_path_buf(), content)
            }
        }
        Err(err) => {
            tracing::error!("chafa failed: {:?}", err);
            Preview::None(path.to_path_buf())
        }
    }
}

fn strip_non_sgr_escape_sequences(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\x1b' {
            result.push(c);
            continue;
        }

        if chars.peek() != Some(&'[') {
            result.push(c);
            continue;
        }

        let mut sequence = String::from("\x1b[");
        chars.next();

        loop {
            match chars.next() {
                Some(sc) if sc.is_ascii_alphabetic() => {
                    if sc == 'm' {
                        sequence.push(sc);
                        result.push_str(&sequence);
                    }
                    break;
                }
                Some(sc) => {
                    sequence.push(sc);
                }
                None => break,
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_sgr_sequences() {
        let input = "\x1b[38;2;255;100;50mhello\x1b[0m";
        assert_eq!(strip_non_sgr_escape_sequences(input), input);
    }

    #[test]
    fn strips_cursor_movement_sequences() {
        let input = "\x1b[2Chello";
        assert_eq!(strip_non_sgr_escape_sequences(input), "hello");
    }

    #[test]
    fn strips_cursor_position_sequences() {
        let input = "\x1b[1;1Hhello";
        assert_eq!(strip_non_sgr_escape_sequences(input), "hello");
    }

    #[test]
    fn strips_erase_sequences() {
        let input = "hello\x1b[2J\x1b[Kworld";
        assert_eq!(strip_non_sgr_escape_sequences(input), "helloworld");
    }

    #[test]
    fn mixed_sgr_and_non_sgr() {
        let input = "\x1b[2C\x1b[31mhello\x1b[0m\x1b[1A";
        assert_eq!(
            strip_non_sgr_escape_sequences(input),
            "\x1b[31mhello\x1b[0m"
        );
    }

    #[test]
    fn plain_text_unchanged() {
        let input = "hello world";
        assert_eq!(strip_non_sgr_escape_sequences(input), input);
    }

    #[test]
    fn empty_string_unchanged() {
        assert_eq!(strip_non_sgr_escape_sequences(""), "");
    }

    #[test]
    fn preserves_non_csi_escape_sequences() {
        let input = "\x1b]hello\x1b[31mworld\x1b[0m";
        assert_eq!(
            strip_non_sgr_escape_sequences(input),
            "\x1b]hello\x1b[31mworld\x1b[0m"
        );
    }
}
