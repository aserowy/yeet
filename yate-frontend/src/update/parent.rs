use std::path::Path;

use yate_keymap::message::{Message, ViewPortDirection};

use crate::{
    layout::AppLayout,
    model::{
        buffer::{BufferLine, Cursor, CursorPosition},
        Model,
    },
};

use super::{buffer, path};

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    let path = Path::new(&model.current_path);
    let buffer = &mut model.parent;
    let layout = &layout.parent_directory;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    match path.parent() {
        Some(parent) => {
            buffer.lines = match path::get_directory_content(parent) {
                Ok(content) => content,
                Err(_) => {
                    vec![BufferLine {
                        content: "Error reading directory".to_string(),
                        ..Default::default()
                    }]
                }
            };

            buffer::update(&model.mode, buffer, message);

            let current_filename = match path.file_name() {
                Some(content) => content.to_str(),
                None => None,
            };

            let current_line = match current_filename {
                Some(content) => buffer.lines.iter().position(|line| line.content == content),
                None => None,
            };

            if let Some(index) = current_line {
                if let Some(cursor) = &mut buffer.cursor {
                    cursor.vertical_index = index;
                } else {
                    buffer.cursor = Some(Cursor {
                        horizontial_index: CursorPosition::None,
                        vertical_index: index,
                        ..Default::default()
                    });
                }

                buffer::update(
                    &model.mode,
                    buffer,
                    &Message::MoveViewPort(ViewPortDirection::CenterOnCursor),
                );
            }
        }
        None => {
            buffer.cursor = None;
            buffer.lines = vec![];
            buffer::update(&model.mode, buffer, message);
        }
    }
}
