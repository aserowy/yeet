use crate::{
    message::ViewPortDirection,
    model::{viewport::ViewPort, CursorPosition, TextBuffer},
    view::wrap,
};

pub fn update_by_cursor(viewport: &mut ViewPort, buffer: &TextBuffer) {
    if buffer.lines.is_empty() {
        return;
    }

    let cursor = &viewport.cursor;
    if cursor.vertical_index >= buffer.lines.len() {
        return;
    }

    if viewport.wrap {
        update_by_cursor_wrap(viewport, buffer);
        return;
    }

    let viewport_offset = if viewport.height == 0 {
        0
    } else {
        usize::from(viewport.height - 1)
    };

    if viewport.vertical_index > cursor.vertical_index {
        viewport.vertical_index = cursor.vertical_index;
    } else if viewport.vertical_index + viewport_offset < cursor.vertical_index {
        viewport.vertical_index = cursor.vertical_index - viewport_offset;
    }

    let cursor_index = match cursor.horizontal_index {
        CursorPosition::Absolute {
            current,
            expanded: _,
        } => current,
        CursorPosition::End => {
            let line_lenght = buffer.lines[cursor.vertical_index].len();
            line_lenght.saturating_sub(1)
        }
        CursorPosition::None => return,
    };

    let line = &buffer.lines[cursor.vertical_index];
    if viewport.horizontal_index > cursor_index {
        viewport.horizontal_index = cursor_index;
    } else if viewport.horizontal_index + viewport.get_content_width(line) <= cursor_index {
        viewport.horizontal_index =
            cursor_index.saturating_sub(viewport.get_content_width(line)) + 1;
    }
}

fn update_by_cursor_wrap(viewport: &mut ViewPort, buffer: &TextBuffer) {
    viewport.horizontal_index = 0;

    let height = usize::from(viewport.height);
    let cursor_line = viewport.cursor.vertical_index;

    if viewport.vertical_index > cursor_line {
        viewport.vertical_index = cursor_line;
        return;
    }

    let mut visual_rows = 0;
    for i in viewport.vertical_index..=cursor_line {
        if i >= buffer.lines.len() {
            break;
        }
        let line = &buffer.lines[i];
        let line_height = wrap::visual_line_count(&line.content, viewport.get_content_width(line));
        visual_rows += line_height;
    }

    if visual_rows <= height {
        return;
    }

    let cursor_line_height = {
        let line = &buffer.lines[cursor_line];
        wrap::visual_line_count(&line.content, viewport.get_content_width(line))
    };

    let mut available = height.saturating_sub(cursor_line_height);
    let mut new_start = cursor_line;

    for i in (viewport.vertical_index..cursor_line).rev() {
        if i >= buffer.lines.len() {
            continue;
        }
        let line = &buffer.lines[i];
        let line_height = wrap::visual_line_count(&line.content, viewport.get_content_width(line));
        if line_height > available {
            break;
        }
        available -= line_height;
        new_start = i;
    }

    viewport.vertical_index = new_start;
}

pub fn update_by_direction(
    viewport: &mut ViewPort,
    buffer: &TextBuffer,
    direction: &ViewPortDirection,
) {
    if buffer.lines.is_empty() {
        return;
    }

    if viewport.cursor.vertical_index >= buffer.lines.len() {
        return;
    }

    match direction {
        ViewPortDirection::BottomOnCursor => {
            if viewport.cursor.vertical_index < usize::from(viewport.height) {
                viewport.vertical_index = 0;
            } else {
                let index = viewport.cursor.vertical_index - usize::from(viewport.height) + 1;
                viewport.vertical_index = index;
            }
        }
        ViewPortDirection::CenterOnCursor => {
            let index_offset = viewport.height / 2;
            if viewport.cursor.vertical_index < usize::from(index_offset) {
                viewport.vertical_index = 0;
            } else {
                viewport.vertical_index =
                    viewport.cursor.vertical_index - usize::from(index_offset);
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

            viewport.cursor.vertical_index = viewport
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

            viewport.cursor.vertical_index = viewport
                .cursor
                .vertical_index
                .saturating_sub(usize::from(index_offset));
        }
        ViewPortDirection::TopOnCursor => {
            viewport.vertical_index = viewport.cursor.vertical_index;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::update_by_cursor;
    use crate::model::{viewport::ViewPort, Cursor, TextBuffer};

    #[test]
    fn update_by_cursor_ignores_out_of_bounds_cursor() {
        let mut viewport = ViewPort {
            cursor: Cursor {
                vertical_index: 10,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut buffer = TextBuffer::default();
        buffer.lines.clear();
        buffer.lines.push(Default::default());

        update_by_cursor(&mut viewport, &buffer);
    }

    #[test]
    fn wrap_scrolls_to_show_cursor_line() {
        let mut viewport = ViewPort {
            width: 10,
            height: 5,
            wrap: true,
            cursor: Cursor {
                vertical_index: 1,
                ..Default::default()
            },
            vertical_index: 0,
            ..Default::default()
        };

        let mut buffer = TextBuffer::default();
        buffer.lines.clear();
        buffer.lines.push(crate::model::BufferLine::from(
            "this is a long line that wraps multiple times in the viewport",
        ));
        buffer
            .lines
            .push(crate::model::BufferLine::from("second line"));

        update_by_cursor(&mut viewport, &buffer);

        assert!(
            viewport.vertical_index <= viewport.cursor.vertical_index,
            "viewport should scroll to show cursor line"
        );
    }

    #[test]
    fn wrap_forces_horizontal_index_to_zero() {
        let mut viewport = ViewPort {
            width: 10,
            height: 10,
            wrap: true,
            horizontal_index: 5,
            cursor: Cursor {
                vertical_index: 0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut buffer = TextBuffer::default();
        buffer.lines.clear();
        buffer
            .lines
            .push(crate::model::BufferLine::from("hello world"));

        update_by_cursor(&mut viewport, &buffer);

        assert_eq!(viewport.horizontal_index, 0);
    }

    #[test]
    fn wrap_cursor_above_viewport_scrolls_up() {
        let mut viewport = ViewPort {
            width: 10,
            height: 5,
            wrap: true,
            cursor: Cursor {
                vertical_index: 0,
                ..Default::default()
            },
            vertical_index: 2,
            ..Default::default()
        };

        let mut buffer = TextBuffer::default();
        buffer.lines.clear();
        for _ in 0..5 {
            buffer.lines.push(crate::model::BufferLine::from("short"));
        }

        update_by_cursor(&mut viewport, &buffer);

        assert_eq!(viewport.vertical_index, 0);
    }
}
