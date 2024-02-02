use yate_keymap::message::Message;

use crate::{
    event::PostRenderAction,
    layout::AppLayout,
    model::{
        buffer::{BufferChanged, BufferLine, Cursor},
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

    if let Some(modifications) = buffer::update(&model.mode, buffer, message) {
        let path = &model.current_path;

        let mut tasks = Vec::new();
        for modification in modifications {
            match modification {
                BufferChanged::LineDeleted(_, name) => {
                    tasks.push(PostRenderAction::Task(Task::DeleteFile(path.join(name))))
                }
            }
        }

        Some(tasks)
    } else {
        None
    }
}
