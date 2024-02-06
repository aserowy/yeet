use yate_keymap::message::{Message, Mode};

use crate::{
    event::PostRenderAction,
    layout::AppLayout,
    model::{
        buffer::{undo::BufferChanged, BufferLine, BufferResult, Cursor},
        Model,
    },
    task::Task,
};

use super::{buffer, path};

pub fn update(
    model: &mut Model,
    layout: &AppLayout,
    message: &Message,
) -> Option<Vec<PostRenderAction>> {
    let buffer = &mut model.current_directory;
    let layout = &layout.current_directory;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    buffer::update(&model.mode, buffer, message);

    None
}

pub fn save_changes(model: &mut Model) -> Option<Vec<PostRenderAction>> {
    if let Some(result) = buffer::update(
        &model.mode,
        &mut model.current_directory,
        &Message::SaveBuffer(None),
    ) {
        let path = &model.current_path;

        let mut tasks = Vec::new();
        if let BufferResult::Changes(modifications) = result {
            // TODO: consolidate changes to enable 1:1 tasks (relevant when line navigation in insert is possible)
            for modification in modifications {
                match modification {
                    BufferChanged::LineAdded(_, name) => {
                        tasks.push(PostRenderAction::Task(Task::AddPath(path.join(name))))
                    }
                    BufferChanged::LineRemoved(_, name) => {
                        tasks.push(PostRenderAction::Task(Task::DeletePath(path.join(name))))
                    }
                    BufferChanged::Content(_, old_name, new_name) => {
                        tasks.push(PostRenderAction::Task(Task::RenamePath(
                            path.join(old_name),
                            path.join(new_name),
                        )))
                    }
                }
            }
        }

        Some(tasks)
    } else {
        None
    }
}

pub fn set_content(model: &mut Model) {
    if model.mode != Mode::Insert {
        let buffer = &mut model.current_directory;
        // TODO: remove when notify is implemented
        // FIX: shows no content in preview when uncommited changes are present
        buffer.lines = match path::get_directory_content(&model.current_path) {
            Ok(content) => {
                if buffer.cursor.is_none() {
                    buffer.cursor = Some(Cursor::default());
                }

                content
            }
            Err(_) => {
                buffer.cursor = None;

                vec![BufferLine {
                    content: "Error reading directory".to_string(),
                    ..Default::default()
                }]
            }
        };
    }
}
