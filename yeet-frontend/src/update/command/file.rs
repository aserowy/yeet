use std::path::{Path, PathBuf};

use yeet_keymap::message::KeymapMessage;

use crate::{
    action::{self, Action},
    event::Message,
    model::{mark::Marks, Buffer, DirectoryBuffer},
    task::Task,
    update::app,
};

pub fn copy_path(marks: &Marks, source_path: &Path, target: &str) -> Vec<Action> {
    tracing::info!("copying path: {:?}", source_path);
    match get_target_file_path(marks, target, source_path) {
        Ok(target_path) => {
            vec![Action::Task(Task::CopyPath(
                source_path.to_path_buf(),
                target_path,
            ))]
        }
        Err(err) => {
            vec![Action::EmitMessages(vec![Message::Error(err)])]
        }
    }
}

pub fn delete_selection(buffer: &DirectoryBuffer) -> Vec<Action> {
    let mut actions = Vec::new();
    if let Some(path) = &buffer.resolve_path() {
        tracing::info!("deleting path: {:?}", path);
        actions.push(Action::Task(Task::DeletePath(path.to_path_buf())));
    } else {
        tracing::warn!("deleting path failed: no path in preview set");
    }

    actions
}

pub fn rename_path(marks: &Marks, source_path: &Path, target: &str) -> Vec<Action> {
    tracing::info!("renaming path: {:?}", source_path);
    match get_target_file_path(marks, target, source_path) {
        Ok(target_path) => {
            vec![Action::Task(Task::RenamePath(
                source_path.to_path_buf(),
                target_path,
            ))]
        }
        Err(err) => {
            vec![Action::EmitMessages(vec![Message::Error(err)])]
        }
    }
}

pub fn refresh(app: &mut crate::model::App) -> Vec<Action> {
    let (_, current, preview) = app::directory_buffers(app);
    let preview_path = match preview {
        Buffer::Directory(buffer) => buffer.resolve_path(),
        Buffer::Image(buffer) => buffer.resolve_path(),
        Buffer::Content(_) => None,
        Buffer::Empty => None,
    };

    let navigation = if let Some(path) = preview_path {
        KeymapMessage::NavigateToPathAsPreview(path.to_path_buf())
    } else if let Buffer::Directory(buffer) = current {
        KeymapMessage::NavigateToPath(buffer.path.clone())
    } else {
        return Vec::new();
    };

    vec![action::emit_keymap(navigation)]
}

fn get_target_file_path(
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

    let source_parent = match source_path.parent() {
        Some(it) => it,
        None => {
            return Err(format!(
                "could not resolve parent from path {:?}",
                source_path
            ))
        }
    };

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
        expand_target_path(target, source_parent)
    };

    let target_file = target_dir.join(file_name);

    tracing::debug!(
        source = %source_path.display(),
        target_dir = %target_dir.display(),
        target_file = %target_file.display(),
        "resolved target file path"
    );

    if target_dir.is_dir() && target_dir.exists() && !target_file.exists() {
        Ok(target_file)
    } else {
        Err("target path is not valid".to_string())
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
