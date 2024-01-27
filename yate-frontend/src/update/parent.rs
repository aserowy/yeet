use std::path::Path;

use yate_keymap::message::{Message, ViewPortDirection};

use crate::{
    layout::AppLayout,
    model::{
        buffer::{Cursor, CursorPosition},
        Model,
    },
};

use super::{buffer, path};

pub fn update(model: &mut Model, layout: &AppLayout, message: &Message) {
    let path = Path::new(&model.current_path);
    let buffer = &mut model.parent_directory;
    let layout = &layout.parent_directory;

    super::set_viewport_dimensions(&mut buffer.view_port, layout);

    match path.parent() {
        Some(parent) => {
            buffer.lines = path::get_directory_content(parent);

            buffer::update(buffer, message);

            let current_filename = path.file_name().unwrap().to_str().unwrap();
            let current_line = buffer
                .lines
                .iter()
                .position(|line| line.content == current_filename);

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
                    buffer,
                    &Message::MoveViewPort(ViewPortDirection::CenterOnCursor),
                );
            }
        }
        None => {
            buffer.cursor = None;
            buffer.lines = vec![];
            buffer::update(buffer, message);
        }
    }
}
