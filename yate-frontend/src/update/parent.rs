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
    match path.parent() {
        Some(parent) => {
            path::update_buffer_with_path(
                &mut model.parent_directory,
                &layout.parent_directory,
                message,
                parent,
            );

            let current_filename = path.file_name().unwrap().to_str().unwrap();
            if let Some(index) = model
                .parent_directory
                .lines
                .iter()
                .position(|line| line.content == current_filename)
            {
                if let Some(cursor) = &mut model.parent_directory.cursor {
                    cursor.vertical_index = index;
                } else {
                    model.parent_directory.cursor = Some(Cursor {
                        horizontial_index: CursorPosition::None,
                        vertical_index: index,
                        ..Default::default()
                    });
                }
            }

            buffer::update(
                &mut model.parent_directory,
                &Message::MoveViewPort(ViewPortDirection::CenterOnCursor),
            );
        }
        None => model.parent_directory.lines = vec![],
    }
}
