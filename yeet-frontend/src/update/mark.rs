use crate::{
    action::Action,
    error::AppError,
    model::{
        self,
        mark::{Marks, MARK_SIGN_ID},
        App, Buffer,
    },
    task::Task,
    theme::Theme,
};

use super::{app, sign};

pub fn add(
    app: &mut App,
    marks: &mut Marks,
    char: char,
    theme: &Theme,
) -> Result<Vec<Action>, AppError> {
    let (window, contents) = app.current_window_and_contents_mut()?;
    let (vp, buffer) = match app::get_focused_current_mut(window, contents)? {
        (vp, Buffer::Directory(it)) => (vp, it),
        (_, Buffer::Image(_))
        | (_, Buffer::Content(_))
        | (_, Buffer::PathReference(_))
        | (_, Buffer::Tasks(_))
        | (_, Buffer::QuickFix(_))
        | (_, Buffer::Empty) => return Ok(Vec::new()),
    };

    let selected = model::get_selected_path(buffer, &vp.cursor);
    if let Some(selected) = selected {
        let removed = marks.entries.insert(char, selected.clone());
        if let Some(removed) = removed {
            sign::unset_sign_for_paths(
                app.contents.buffers.values_mut().collect(),
                vec![removed],
                MARK_SIGN_ID,
            );
        }

        sign::set_sign_for_paths(
            app.contents.buffers.values_mut().collect(),
            vec![selected],
            MARK_SIGN_ID,
            theme,
        );
    }

    Ok(Vec::new())
}

pub fn delete(marks: &mut Marks, buffers: Vec<&mut Buffer>, delete: &Vec<char>) -> Vec<Action> {
    let mut persisted = Vec::new();
    let mut paths = Vec::new();
    for mark in delete {
        let deleted = marks.entries.remove_entry(mark);
        if let Some((mark, path)) = deleted {
            persisted.push(mark);
            paths.push(path);
        }
    }

    sign::unset_sign_for_paths(buffers, paths, MARK_SIGN_ID);

    if persisted.is_empty() {
        Vec::new()
    } else {
        vec![Action::Task(Task::DeleteMarks(persisted))]
    }
}
