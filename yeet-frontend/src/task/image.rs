use std::{path::Path, process::Stdio, str};

use ratatui::layout::Rect;
use tokio::process::Command;

use crate::model::emulator::Emulator;

#[tracing::instrument]
pub async fn load<'a>(emulator: &Emulator, path: &Path, rect: &Rect) -> Option<String> {
    match emulator {
        // kitty
        Emulator::Ghostty | Emulator::Kitty | Emulator::Tabby => {
            // TODO: check for zellij and fallback to chafa
            load_with_chafa(path, rect).await

            // TODO: adding kitty protocol
        }
        // sixel
        Emulator::Foot
        | Emulator::Hyper
        | Emulator::Iterm2
        | Emulator::VsCode
        | Emulator::WezTerm
        | Emulator::WindowsTerminal => load_with_sixel(path, rect).await,
        // chafa
        Emulator::Alacritty | Emulator::Neovim | Emulator::Unknown | Emulator::Urxvt => {
            load_with_chafa(path, rect).await
        }
    }
}

async fn load_with_sixel(path: &Path, rect: &Rect) -> Option<String> {
    // TODO: add sixel
    let sample = "
        \u{1b}Pq
        \"2;1;100;200
        #0;2;0;0;0#1;2;100;100;0#2;2;0;100;0
        #1~~@@vv@@~~@@~~$
        #2??}}GG}}??}}??-
        #1!14@
        \u{1b}\\
    "
    .to_string();

    Some(sample)
}

async fn load_with_chafa(path: &Path, rect: &Rect) -> Option<String> {
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
            } else if output.stdout.is_empty() {
                tracing::warn!("chafa failed: image result is empty");
            }

            str::from_utf8(&output.stdout).ok().map(|s| s.to_string())
        }
        Err(err) => {
            tracing::error!("chafa failed: {:?}", err);
            None
        }
    }
}
