#![allow(unused_imports)]

use std::{
    ffi::{OsStr, OsString},
    io,
    path::Path,
    process::ExitStatus,
};

use tokio::process::Command;

#[cfg(all(unix, not(target_os = "macos")))]
pub async fn path(path: &Path) -> Result<ExitStatus, io::Error> {
    use std::{env, path::PathBuf};

    use tokio::fs;

    async fn is_wsl() -> bool {
        if std::env::consts::OS != "linux" {
            return false;
        }
        if let Ok(osrelease) = fs::read_to_string("/proc/sys/kernel/osrelease").await {
            if osrelease.to_lowercase().contains("microsoft") {
                return !is_docker().await;
            }
        }
        if let Ok(version) = fs::read_to_string("/proc/version").await {
            if version.to_lowercase().contains("microsoft") {
                return !is_docker().await;
            }
        }

        false
    }

    async fn is_docker() -> bool {
        match fs::read_to_string("/proc/self/cgroup").await {
            Ok(file_contents) => {
                file_contents.contains("docker") || fs::metadata("/.dockerenv").await.is_ok()
            }
            Err(_error) => false || fs::metadata("/.dockerenv").await.is_ok(),
        }
    }

    fn wsl_path<T: AsRef<OsStr>>(path: T) -> OsString {
        fn path_relative_to_current_dir<T: AsRef<OsStr>>(path: T) -> Option<PathBuf> {
            let path = Path::new(&path);
            if path.is_relative() {
                return None;
            }

            let base = env::current_dir().ok()?;
            pathdiff::diff_paths(path, base)
        }

        match path_relative_to_current_dir(&path) {
            None => OsString::from(&path),
            Some(relative) => OsString::from(relative),
        }
    }

    let mut cmd = if is_wsl().await {
        Command::new("wslview").arg(wsl_path(path)).spawn()?
    } else {
        Command::new("xdg-open").arg(path).spawn()?
    };
    cmd.wait().await
}

#[cfg(target_os = "macos")]
pub async fn path(path: &Path) -> Result<ExitStatus, io::Error> {
    Command::new("/usr/bin/open")
        .arg(path)
        .spawn()?
        .wait()
        .await
}

#[cfg(target_os = "redox")]
pub async fn path(path: &Path) -> Result<ExitStatus, io::Error> {
    Command::new("/ui/bin/launcher")
        .arg(path)
        .spawn()?
        .wait()
        .await
}

#[cfg(windows)]
pub async fn path(path: &Path) -> Result<ExitStatus, io::Error> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    fn wrap_in_quotes<T: AsRef<OsStr>>(path: T) -> String {
        let mut result = OsString::from("\"");
        result.push(path);
        result.push("\"");

        result.to_string_lossy().to_string()
    }

    Command::new("cmd")
        .args(&["/c", "start", "\"\"", &wrap_in_quotes(path)])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?
        .wait()
        .await
}

#[cfg(not(any(unix, windows, target_os = "redox")))]
compile_error!("open is not supported on this platform");
