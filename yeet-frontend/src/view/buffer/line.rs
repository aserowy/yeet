use yeet_keymap::message::Mode;

use crate::model::buffer::{
    viewport::ViewPort, BufferLine, Cursor, CursorPosition, StylePartial, StylePartialSpan,
};

use super::style::{
    CURSORLINE_STYLE_PARTIAL, CURSOR_COMMAND_STYLE_PARTIAL, CURSOR_INSERT_STYLE_PARTIAL,
    CURSOR_NORMAL_STYLE_PARTIAL,
};

pub fn get_cursor_style_partials(
    vp: &ViewPort,
    mode: &Mode,
    cursor: &Option<Cursor>,
    index: &usize,
    line: &BufferLine,
) -> Vec<StylePartialSpan> {
    if let Some(cursor) = cursor {
        if cursor.vertical_index - vp.vertical_index != *index {
            return Vec::new();
        }

        let offset = vp.get_offset_width(line);
        let content_width = vp.width - offset;

        let line = &line.content[vp.horizontal_index..];
        let chars_count = line.chars().count();

        let line_length = if chars_count > content_width {
            content_width
        } else if chars_count == 0 {
            1
        } else {
            chars_count
        };

        let mut spans = Vec::new();
        if !cursor.hide_cursor_line {
            spans.push((offset, vp.width, CURSORLINE_STYLE_PARTIAL.clone()));
        }

        if cursor.hide_cursor {
            return spans;
        }

        let cursor_index = match &cursor.horizontial_index {
            CursorPosition::End => line_length - vp.horizontal_index - 1,
            CursorPosition::None => return spans,
            CursorPosition::Absolute {
                current,
                expanded: _,
            } => *current - vp.horizontal_index,
        };

        spans.push((
            offset + cursor_index,
            offset + cursor_index + 1,
            get_partial_style(mode),
        ));

        spans
    } else {
        Vec::new()
    }
}

fn get_partial_style(mode: &Mode) -> StylePartial {
    match mode {
        Mode::Command => CURSOR_COMMAND_STYLE_PARTIAL.clone(),
        Mode::Insert => CURSOR_INSERT_STYLE_PARTIAL.clone(),
        Mode::Navigation => CURSOR_NORMAL_STYLE_PARTIAL.clone(),
        Mode::Normal => CURSOR_NORMAL_STYLE_PARTIAL.clone(),
    }
}