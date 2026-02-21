use crate::{
    message::ViewPortDirection,
    model::{viewport::ViewPort, CursorPosition, TextBuffer},
};

pub fn update_by_cursor(viewport: &mut ViewPort, buffer: &TextBuffer) {
    if buffer.lines.is_empty() {
        return;
    }

    if buffer.cursor.vertical_index >= buffer.lines.len() {
        return;
    }

    let viewport_offset = if viewport.height == 0 {
        0
    } else {
        usize::from(viewport.height - 1)
    };

    if viewport.vertical_index > buffer.cursor.vertical_index {
        viewport.vertical_index = buffer.cursor.vertical_index;
    } else if viewport.vertical_index + viewport_offset < buffer.cursor.vertical_index {
        viewport.vertical_index = buffer.cursor.vertical_index - viewport_offset;
    }

    let cursor_index = match buffer.cursor.horizontal_index {
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => current,
        CursorPosition::End => {
            let line_lenght = buffer.lines[buffer.cursor.vertical_index].len();
            line_lenght.saturating_sub(1)
        }
        CursorPosition::None => return,
    };

    let line = &buffer.lines[buffer.cursor.vertical_index];
    if viewport.horizontal_index > cursor_index {
        viewport.horizontal_index = cursor_index;
    } else if viewport.horizontal_index + viewport.get_content_width(line) <= cursor_index {
        viewport.horizontal_index =
            cursor_index.saturating_sub(viewport.get_content_width(line)) + 1;
    }
}

pub fn update_by_direction(
    viewport: &mut ViewPort,
    buffer: &mut TextBuffer,
    direction: &ViewPortDirection,
) {
    if buffer.lines.is_empty() {
        return;
    }

    if buffer.cursor.vertical_index >= buffer.lines.len() {
        return;
    }

    match direction {
        ViewPortDirection::BottomOnCursor => {
            if buffer.cursor.vertical_index < usize::from(viewport.height) {
                viewport.vertical_index = 0;
            } else {
                let index = buffer.cursor.vertical_index - usize::from(viewport.height) + 1;
                viewport.vertical_index = index;
            }
        }
        ViewPortDirection::CenterOnCursor => {
            let index_offset = viewport.height / 2;
            if buffer.cursor.vertical_index < usize::from(index_offset) {
                viewport.vertical_index = 0;
            } else {
                viewport.vertical_index = buffer.cursor.vertical_index - usize::from(index_offset);
            }
        }
        ViewPortDirection::HalfPageDown => {
            let index_offset = viewport.height / 2;
            let viewport_end_index = viewport.vertical_index + usize::from(viewport.height - 1);
            let viewport_end_after_move_index = viewport_end_index + usize::from(index_offset);

            if viewport_end_after_move_index < buffer.lines.len() {
                viewport.vertical_index += usize::from(index_offset);
            } else if usize::from(viewport.height) > buffer.lines.len() {
                viewport.vertical_index = 0;
            } else {
                viewport.vertical_index = buffer.lines.len() - usize::from(viewport.height);
            }

            buffer.cursor.vertical_index = buffer
                .cursor
                .vertical_index
                .checked_add(usize::from(index_offset))
                .filter(|index| *index < buffer.lines.len())
                .unwrap_or_else(|| buffer.lines.len() - 1);
        }
        ViewPortDirection::HalfPageUp => {
            let index_offset = viewport.height / 2;
            if viewport.vertical_index < usize::from(index_offset) {
                viewport.vertical_index = 0;
            } else {
                viewport.vertical_index -= usize::from(index_offset);
            }

            buffer.cursor.vertical_index = buffer
                .cursor
                .vertical_index
                .saturating_sub(usize::from(index_offset));
        }
        ViewPortDirection::TopOnCursor => {
            viewport.vertical_index = buffer.cursor.vertical_index;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::update_by_cursor;
    use crate::model::{viewport::ViewPort, TextBuffer};

    #[test]
    fn update_by_cursor_ignores_out_of_bounds_cursor() {
        let mut viewport = ViewPort::default();
        let mut buffer = TextBuffer::default();
        buffer.lines.clear();
        buffer.lines.push(Default::default());

        buffer.cursor.vertical_index = 10;

        update_by_cursor(&mut viewport, &buffer);
    }
}
