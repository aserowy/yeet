#![allow(unused_imports)]

use std::{
    ffi::{OsStr, OsString},
    io,
    path::Path,
    process::ExitStatus,
};

use tokio::process::Command;

// pub fn that(path: impl AsRef<OsStr>) -> io::Result<()> {
//     let mut last_err = None;
//     for mut cmd in commands(path) {
//         match cmd.status_without_output() {
//             Ok(status) => {
//                 return Ok(status).into_result(&cmd);
//             }
//             Err(err) => last_err = Some(err),
//         }
//     }
//     Err(last_err.expect("no launcher worked, at least one error"))
// }

// pub fn commands<T: AsRef<OsStr>>(path: T) -> Vec<Command> {
//     let path = path.as_ref();
//     let mut commands: Vec<(&str, Vec<&OsStr>)> = vec![];
//
//     let wsl_path = wsl_path(path);
// CUSTOM DEPENDENCY: is_wsl
//     if is_wsl::is_wsl() {
//         commands.push(("wslview", vec![&wsl_path]));
//     }
//
//     commands.extend_from_slice(&[
//         ("xdg-open", vec![&path]),
//         ("gio", vec![OsStr::new("open"), path]),
//         ("gnome-open", vec![path]),
//         ("kde-open", vec![path]),
//     ]);
//
//     commands
//         .iter()
//         .map(|(command, args)| {
//             let mut cmd = Command::new(command);
//             cmd.args(args);
//             cmd
//         })
//         .collect()
// }

// fn wsl_path<T: AsRef<OsStr>>(path: T) -> OsString {
//     fn path_relative_to_current_dir<T: AsRef<OsStr>>(path: T) -> Option<PathBuf> {
//         let path = Path::new(&path);
//
//         if path.is_relative() {
//             return None;
//         }
//
//         let base = env::current_dir().ok()?;
//         pathdiff::diff_paths(path, base)
//     }
//
//     match path_relative_to_current_dir(&path) {
//         None => OsString::from(&path),
//         Some(relative) => OsString::from(relative),
//     }
// }

// TODO: add wsl support
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "solaris"
))]
pub async fn path(path: &Path) -> Result<ExitStatus, io::Error> {
    let mut cmd = Command::new("xdg-open")
        .arg(path)
        .spawn()
        .expect("Failed to open file");

    cmd.wait().await
}

#[cfg(target_os = "macos")]
pub async fn path(path: &Path) -> Result<ExitStatus, io::Error> {
    let mut cmd = Command::new("/usr/bin/open")
        .arg(path)
        .spawn()
        .expect("Failed to open file");

    cmd.wait().await
}

#[cfg(target_os = "redox")]
pub async fn path(path: &Path) -> Result<ExitStatus, io::Error> {
    let mut cmd = Command::new("/ui/bin/launcher")
        .arg(path)
        .spawn()
        .expect("Failed to open file");

    cmd.wait().await
}

#[cfg(target_os = "windows")]
pub async fn path(path: &Path) -> Result<ExitStatus, io::Error> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    fn wrap_in_quotes<T: AsRef<OsStr>>(path: T) -> OsString {
        let mut result = OsString::from("\"");
        result.push(path);
        result.push("\"");

        result
    }

    let mut cmd = Command::new("cmd")
        .arg("/c")
        .arg("start")
        .raw_arg("\"\"")
        .raw_arg(wrap_in_quotes(path))
        .creation_flags(CREATE_NO_WINDOW);

    // ... spawn n stuff
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "solaris",
    target_os = "ios",
    target_os = "macos",
    target_os = "windows",
    target_os = "redox"
)))]
compile_error!("open is not supported on this platform");
