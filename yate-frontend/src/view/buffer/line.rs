use yate_keymap::message::Mode;

use crate::model::buffer::{
    BufferLine, Cursor, CursorPosition, StylePartial, StylePartialSpan, ViewPort,
};

use super::style::{
    CURSORLINE_STYLE_PARTIAL, CURSOR_COMMAND_STYLE_PARTIAL, CURSOR_NORMAL_STYLE_PARTIAL,
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
        let line_length = if line.chars().count() > content_width {
            content_width
        } else {
            let length = line.chars().count();
            if length == 0 {
                1
            } else {
                length
            }
        };

        let mut spans = Vec::new();
        if !cursor.hide_cursor_line {
            spans.push((offset, vp.width, CURSORLINE_STYLE_PARTIAL.clone()));
        }

        if cursor.hide_cursor {
            return spans;
        }

        let cursor_index = match &cursor.horizontial_index {
            CursorPosition::Absolute(i) => {
                let corrected_index = *i - vp.horizontal_index;
                if corrected_index < line_length {
                    corrected_index
                } else {
                    line_length - get_cursor_offset(mode)
                }
            }
            CursorPosition::End => line_length - vp.horizontal_index - 1,
            CursorPosition::None => return spans,
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

fn get_cursor_offset(mode: &Mode) -> usize {
    match mode {
        Mode::Normal => 1,
        Mode::Command => 0,
    }
}

fn get_partial_style(mode: &Mode) -> StylePartial {
    match mode {
        Mode::Normal => CURSOR_NORMAL_STYLE_PARTIAL.clone(),
        Mode::Command => CURSOR_COMMAND_STYLE_PARTIAL.clone(),
    }
}
