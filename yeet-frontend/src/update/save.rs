use yeet_buffer::{
    message::BufferMessage,
    model::{
        undo::{consolidate_modifications, BufferChanged},
        viewport::ViewPort,
        BufferResult, Mode,
    },
};

use crate::{
    action::Action,
    error::AppError,
    model::{junkyard::JunkYard, App, Buffer, Contents, DirectoryBuffer, Window},
    task::Task,
};

use super::{app, junkyard::trash_to_junkyard};

#[tracing::instrument(skip(app))]
pub fn current(app: &mut App, junk: &mut JunkYard, mode: &Mode) -> Result<Vec<Action>, AppError> {
    let (window, contents) = app.current_window_and_contents_mut()?;
    let (vp, buffer) = match app::get_focused_current_mut(window, contents)? {
        (vp, Buffer::Directory(it)) => (vp, it),
        (_vp, Buffer::Image(_))
        | (_vp, Buffer::Content(_))
        | (_vp, Buffer::PathReference(_))
        | (_vp, Buffer::Tasks(_))
        | (_vp, Buffer::QuickFix(_))
        | (_vp, Buffer::Help(_))
        | (_vp, Buffer::Empty) => return Ok(Vec::new()),
    };

    Ok(save_directory_buffer(Some(vp), buffer, junk, mode))
}

pub fn all(
    window: &mut Window,
    contents: &mut Contents,
    junk: &mut JunkYard,
    mode: &Mode,
) -> Vec<Action> {
    let dir_ids: Vec<usize> = contents
        .buffers
        .iter()
        .filter_map(|(id, buf)| matches!(buf, Buffer::Directory(_)).then_some(*id))
        .collect();

    let mut actions = Vec::new();
    for id in dir_ids {
        let vp = app::get_viewport_by_buffer_id_mut(window, id);
        if let Some(Buffer::Directory(dir)) = contents.buffers.get_mut(&id) {
            actions.extend(save_directory_buffer(vp, dir, junk, mode));
        }
    }
    actions
}

fn save_directory_buffer(
    viewport: Option<&mut ViewPort>,
    buffer: &mut DirectoryBuffer,
    junk: &mut JunkYard,
    mode: &Mode,
) -> Vec<Action> {
    let selection = viewport
        .as_deref()
        .map(|vp| vp.cursor.vertical_index)
        .and_then(|idx| {
            buffer
                .buffer
                .lines
                .get(idx)
                .map(|line| line.content.clone())
        });

    let mut content: Vec<_> = buffer.buffer.lines.drain(..).collect();
    content.retain(|line| !line.content.is_empty());

    let mut viewport = viewport;

    let message = BufferMessage::SetContent(content);
    yeet_buffer::update(
        viewport.as_deref_mut(),
        mode,
        &mut buffer.buffer,
        std::slice::from_ref(&message),
    );

    if let Some(selection) = selection {
        let message = BufferMessage::SetCursorToLineContent(selection.to_stripped_string());
        yeet_buffer::update(
            viewport.as_deref_mut(),
            mode,
            &mut buffer.buffer,
            std::slice::from_ref(&message),
        );
    }

    let message = BufferMessage::SaveBuffer;
    let result = yeet_buffer::update(
        viewport,
        mode,
        &mut buffer.buffer,
        std::slice::from_ref(&message),
    );

    let mut actions = Vec::new();
    for br in result {
        if let BufferResult::Changes(modifications) = br {
            let path = &buffer.path;
            let mut trashes = Vec::new();
            for modification in consolidate_modifications(&modifications) {
                match modification {
                    BufferChanged::LineAdded(_, name) => {
                        if !name.is_empty() {
                            actions.push(Action::Task(Task::AddPath(
                                path.join(name.to_stripped_string()),
                            )))
                        }
                    }
                    BufferChanged::LineRemoved(_, name) => {
                        trashes.push(path.join(name.to_stripped_string()));
                    }
                    BufferChanged::Content(_, old_name, new_name) => {
                        let task = if new_name.is_empty() {
                            Task::DeletePath(path.join(old_name.to_stripped_string()))
                        } else {
                            Task::RenamePath(
                                path.join(old_name.to_stripped_string()),
                                path.join(new_name.to_stripped_string()),
                            )
                        };
                        actions.push(Action::Task(task));
                    }
                }
            }

            if !trashes.is_empty() {
                let (transaction, obsolete) = trash_to_junkyard(junk, trashes);
                for entry in transaction.entries {
                    actions.push(Action::Task(Task::TrashPath(entry)));
                }

                if let Some(obsolete) = obsolete {
                    for entry in obsolete.entries {
                        actions.push(Action::Task(Task::DeleteJunkYardEntry(entry)));
                    }
                }
            }
        }
    }
    actions
}
