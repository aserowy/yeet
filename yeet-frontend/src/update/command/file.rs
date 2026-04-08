use std::path::{Path, PathBuf};

use yeet_keymap::message::KeymapMessage;

use crate::{
    action::{self, Action},
    error::AppError,
    event::{LogSeverity, Message},
    model::{mark::Marks, App},
    task::Task,
    update::app,
};

pub fn copy_path(marks: &Marks, source_path: &Path, target: &str) -> Vec<Action> {
    tracing::info!("copying path: {:?}", source_path);
    match expand_and_validate_path(marks, target, source_path) {
        Ok(target_path) => {
            vec![Action::Task(Task::CopyPath(
                source_path.to_path_buf(),
                target_path,
            ))]
        }
        Err(err) => {
            vec![Action::EmitMessages(vec![Message::Log(
                LogSeverity::Error,
                err,
            )])]
        }
    }
}

pub fn rename_path(marks: &Marks, source_path: &Path, target: &str) -> Vec<Action> {
    tracing::info!("renaming path: {:?}", source_path);
    match expand_and_validate_path(marks, target, source_path) {
        Ok(target_path) => {
            vec![Action::Task(Task::RenamePath(
                source_path.to_path_buf(),
                target_path,
            ))]
        }
        Err(err) => {
            vec![Action::EmitMessages(vec![Message::Log(
                LogSeverity::Error,
                err,
            )])]
        }
    }
}

fn expand_and_validate_path(
    marks: &Marks,
    target: &str,
    source_path: &Path,
) -> Result<PathBuf, String> {
    let file_name = match source_path.file_name() {
        Some(it) => it,
        None => {
            return Err(format!(
                "could not resolve file name from path {:?}",
                source_path
            ))
        }
    };

    let target_path = expand_path(marks, target, source_path)?;
    let target_file = target_path.join(file_name);

    // NOTE: cannot use `is_file` here because the target file does not exist yet,
    // so we need to check if the path ends with a file name instead
    if target_file.file_name().is_none() {
        return Err(format!(
            "target path {:?} is not a file",
            target_file.display()
        ));
    }

    if target_file.exists() {
        return Err(format!(
            "target file path {:?} exists already",
            target_file.display()
        ));
    }

    let parent_dir = match target_file.parent() {
        Some(it) => it,
        None => {
            return Err(format!(
                "could not resolve parent directory from path {:?}",
                target_file
            ))
        }
    };

    if !parent_dir.exists() {
        return Err(format!(
            "target directory {:?} does not exist",
            target_file.display()
        ));
    }

    Ok(target_file)
}

pub fn refresh(app: &mut App) -> Result<Vec<Action>, AppError> {
    let (window, contents) = app.current_window_and_contents_mut()?;
    let (_, buffer) = app::get_focused_current_mut(window, contents)?;
    let path = buffer.resolve_path();
    let navigation = if let Some(path) = path {
        KeymapMessage::NavigateToPath(path.to_path_buf())
    } else {
        return Ok(Vec::new());
    };

    Ok(vec![action::emit_keymap(navigation)])
}

pub fn expand_path(marks: &Marks, target: &str, source_path: &Path) -> Result<PathBuf, String> {
    let target_dir = if target.starts_with('\'') {
        let mark = match target.chars().nth(1) {
            Some(it) => it,
            None => return Err("invalid mark format".to_string()),
        };

        if let Some(path) = marks.entries.get(&mark) {
            tracing::trace!(mark = %mark, path = %path.display(), "resolved mark to path");
            path.to_path_buf()
        } else {
            return Err(format!("mark '{}' not found", mark));
        }
    } else {
        let source_directory_path = if source_path.is_dir() {
            source_path
        } else {
            match source_path.parent() {
                Some(it) => it,
                None => {
                    return Err(format!(
                        "could not resolve parent from path {:?}",
                        source_path
                    ))
                }
            }
        };

        expand_target_path(target, source_directory_path)
    };

    tracing::debug!(
        source = %source_path.display(),
        target_dir = %target_dir.display(),
        "resolved target directory path"
    );

    Ok(target_dir)
}

pub fn expand_path_without_source(marks: &Marks, target: &str) -> Result<PathBuf, String> {
    let trimmed = target.trim();
    if trimmed.is_empty() {
        return dirs::home_dir().ok_or_else(|| "Home directory could not be resolved.".to_string());
    }

    if trimmed.starts_with('\'') {
        let mark = match trimmed.chars().nth(1) {
            Some(it) => it,
            None => return Err("invalid mark format".to_string()),
        };

        return marks
            .entries
            .get(&mark)
            .map(|path| path.to_path_buf())
            .ok_or_else(|| format!("mark '{}' not found", mark));
    }

    let path = Path::new(trimmed);
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Err("Relative paths require a directory context.".to_string())
    }
}

fn expand_target_path(target: &str, base_dir: &Path) -> PathBuf {
    let target_path = Path::new(target);
    if target_path.is_absolute() {
        tracing::trace!(
            target = %target,
            "target path is absolute, using as-is"
        );
        target_path.to_path_buf()
    } else {
        let expanded = base_dir.join(target_path);
        tracing::trace!(
            target = %target,
            base_dir = %base_dir.display(),
            expanded = %expanded.display(),
            "expanded relative target path"
        );
        expanded
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        action::Action,
        event::{LogSeverity, Message},
        model::mark::Marks,
        task::Task,
    };

    use super::{copy_path, expand_path, rename_path};

    fn unique_temp_dir() -> PathBuf {
        let mut dir = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        dir.push(format!("yeet_test_{}", nanos));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn expand_directorypath_with_absolute_target() {
        let marks = Marks::default();
        let source = PathBuf::from("/home/user/");
        let target = "/tmp";

        let result = expand_path(&marks, target, &source).expect("expand path");

        assert_eq!(result, PathBuf::from("/tmp/"));
    }

    #[test]
    fn expand_filepath_with_absolute_target() {
        let marks = Marks::default();
        let source = PathBuf::from("/home/user/file.txt");
        let target = "/tmp";

        let result = expand_path(&marks, target, &source).expect("expand path");

        assert_eq!(result, PathBuf::from("/tmp/"));
    }

    #[test]
    fn expand_directorypath_with_relative_target() {
        let marks = Marks::default();
        let source = unique_temp_dir();
        let target = "dest";

        let result = expand_path(&marks, target, &source).expect("expand path");

        let target = source.join(Path::new("dest/"));
        assert_eq!(result, target);
    }

    #[test]
    fn expand_filepath_with_relative_target() {
        let marks = Marks::default();
        let source = PathBuf::from("/home/user/file.txt");
        let target = "dest";

        let result = expand_path(&marks, target, &source).expect("expand path");

        assert_eq!(result, PathBuf::from("/home/user/dest/"));
    }

    #[test]
    fn expand_path_with_mark_target() {
        let mut marks = Marks::default();
        marks.entries.insert('a', PathBuf::from("/var/tmp"));
        let source = PathBuf::from("/home/user/file.txt");

        let result = expand_path(&marks, "'a", &source).expect("expand path");

        assert_eq!(result, PathBuf::from("/var/tmp/"));
    }

    #[test]
    fn expand_path_with_missing_mark_returns_error() {
        let marks = Marks::default();
        let source = PathBuf::from("/home/user/file.txt");

        let error = expand_path(&marks, "'x", &source).expect_err("missing mark");

        assert_eq!(error, "mark 'x' not found");
    }

    #[test]
    fn expand_path_with_invalid_mark_format_returns_error() {
        let marks = Marks::default();
        let source = PathBuf::from("/home/user/file.txt");

        let error = expand_path(&marks, "'", &source).expect_err("invalid mark");

        assert_eq!(error, "invalid mark format");
    }

    #[test]
    fn copy_path_returns_copy_task_on_success() {
        let marks = Marks::default();
        let target_dir = unique_temp_dir();
        let source = target_dir.join("source.txt");
        let target = target_dir.to_string_lossy();

        let actions = copy_path(&marks, &source, target.as_ref());

        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            Action::Task(Task::CopyPath(src, dst))
                if src == &source && dst == &target_dir.join("source.txt")
        ));

        let _ = fs::remove_dir_all(&target_dir);
    }

    #[test]
    fn rename_path_returns_rename_task_on_success() {
        let marks = Marks::default();
        let target_dir = unique_temp_dir();
        let source = target_dir.join("source.txt");
        let target = target_dir.to_string_lossy();

        let actions = rename_path(&marks, &source, target.as_ref());

        assert_eq!(actions.len(), 1);
        assert!(matches!(
            &actions[0],
            Action::Task(Task::RenamePath(src, dst))
                if src == &source && dst == &target_dir.join("source.txt")
        ));

        let _ = fs::remove_dir_all(&target_dir);
    }

    #[test]
    fn copy_path_returns_error_when_target_dir_missing() {
        let marks = Marks::default();
        let mut missing_dir = std::env::temp_dir();
        missing_dir.push("yeet_missing_target_dir");
        let source = PathBuf::from("/home/user/file.txt");

        let actions = copy_path(&marks, &source, missing_dir.to_string_lossy().as_ref());

        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], Action::EmitMessages(messages) if matches!(
                messages.as_slice(),
                [Message::Log(LogSeverity::Error,message)] if message.contains("target directory")
            ))
        );
    }

    #[test]
    fn rename_path_returns_error_when_target_dir_missing() {
        let marks = Marks::default();
        let mut missing_dir = std::env::temp_dir();
        missing_dir.push("yeet_missing_target_dir_rename");
        let source = PathBuf::from("/home/user/file.txt");

        let actions = rename_path(&marks, &source, missing_dir.to_string_lossy().as_ref());

        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], Action::EmitMessages(messages) if matches!(
                messages.as_slice(),
                [Message::Log(LogSeverity::Error,message)] if message.contains("target directory")
            ))
        );
    }
}
