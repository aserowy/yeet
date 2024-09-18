use crate::model::{Buffer, BufferLine, Cursor, CursorPosition};

use super::cursor;

pub fn move_cursor_to_word_start_forward(model: &mut Buffer, is_upper: bool) {
    let cursor = match &mut model.cursor {
        Some(cursor) => cursor,
        None => return,
    };

    let current = match model.lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return,
    };

    let index = match cursor::get_horizontal_index(&cursor.horizontal_index, current) {
        Some(index) => index,
        None => return,
    };

    let content = current
        .content
        .to_stripped_string()
        .chars()
        .collect::<Vec<_>>();

    let char = match content.get(index) {
        Some(chr) => chr,
        None => return,
    };

    let is_alphanumeric = char.is_alphanumeric() || char == &'_';
    let predicate = match (is_upper, is_alphanumeric) {
        (true, _) => |c: &char| c.is_whitespace(),
        (false, true) => |c: &char| c == &'_' || c.is_alphanumeric(),
        (false, false) => |c: &char| c != &'_' && !c.is_alphanumeric() && !c.is_whitespace(),
    };

    let next = content
        .iter()
        .enumerate()
        .skip_while(|(i, c)| i <= &index || predicate(c))
        .find(|(_, c)| !c.is_whitespace());

    if let Some((next_index, _)) = next {
        cursor.horizontal_index = CursorPosition::Absolute {
            current: next_index,
            expanded: next_index,
        };
    } else {
        let cursor = match get_cursor_on_word_next_line(cursor, &model.lines) {
            Ok(crsr) => crsr,
            Err(_) => return,
        };

        model.cursor = Some(cursor);
    }
}

pub fn move_cursor_to_word_end_backward(model: &mut Buffer, is_upper: bool) {
    let cursor = match &mut model.cursor {
        Some(cursor) => cursor,
        None => return,
    };

    let current = match model.lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return,
    };

    let content_len = current.content.count_chars();
    let index = match cursor::get_horizontal_index(&cursor.horizontal_index, current) {
        Some(index) => content_len - index - 1,
        None => return,
    };

    let content = current
        .content
        .to_stripped_string()
        .chars()
        .rev()
        .collect::<Vec<_>>();

    let char = match content.get(index) {
        Some(chr) => chr,
        None => return,
    };

    let is_alphanumeric = char.is_alphanumeric() || char == &'_';
    let predicate = match (is_upper, is_alphanumeric) {
        (true, _) => |c: &char| c.is_whitespace(),
        (false, true) => |c: &char| c == &'_' || c.is_alphanumeric(),
        (false, false) => |c: &char| c != &'_' && !c.is_alphanumeric() && !c.is_whitespace(),
    };

    let next = content
        .iter()
        .enumerate()
        .skip_while(|(i, c)| i <= &index || predicate(c))
        .find(|(_, c)| !c.is_whitespace());

    if let Some((next_index, _)) = next {
        let next_index = content_len - next_index - 1;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: next_index,
            expanded: next_index,
        };
    } else {
        let cursor = match get_cursor_on_word_previous_line(cursor, &model.lines) {
            Ok(crsr) => crsr,
            Err(_) => return,
        };

        model.cursor = Some(cursor);
    }
}

pub fn move_cursor_to_word_end_forward(model: &mut Buffer, is_upper: bool) {
    let cursor = match &mut model.cursor {
        Some(cursor) => cursor,
        None => return,
    };

    let current = match model.lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return,
    };

    let index = match cursor::get_horizontal_index(&cursor.horizontal_index, current) {
        Some(index) => index,
        None => return,
    };

    let content = current
        .content
        .to_stripped_string()
        .chars()
        .collect::<Vec<_>>();

    let index = content
        .iter()
        .enumerate()
        .find(|(i, c)| i > &index && !c.is_whitespace());

    if let Some((index, _)) = index {
        let position = get_position_on_word_end(content, index, is_upper);
        if let Ok(position) = position {
            cursor.horizontal_index = position;
        }
    } else {
        let new_line_cursor = match get_cursor_on_word_next_line(cursor, &model.lines) {
            Ok(crsr) => crsr,
            Err(_) => return,
        };

        let current = match model.lines.get(new_line_cursor.vertical_index) {
            Some(line) => line,
            None => return,
        };

        let content = current
            .content
            .to_stripped_string()
            .chars()
            .collect::<Vec<_>>();

        let position = get_position_on_word_end(content, new_line_cursor.vertical_index, is_upper);
        if let Ok(position) = position {
            cursor.vertical_index = new_line_cursor.vertical_index;
            cursor.horizontal_index = position;
        }
    }
}

pub fn move_cursor_to_word_start_backward(model: &mut Buffer, is_upper: bool) {
    let cursor = match &mut model.cursor {
        Some(cursor) => cursor,
        None => return,
    };

    let current = match model.lines.get(cursor.vertical_index) {
        Some(line) => line,
        None => return,
    };

    let content_len = current.content.count_chars();
    let index = match cursor::get_horizontal_index(&cursor.horizontal_index, current) {
        Some(index) => content_len - index - 1,
        None => return,
    };

    let content = current
        .content
        .to_stripped_string()
        .chars()
        .rev()
        .collect::<Vec<_>>();

    let index = content
        .iter()
        .enumerate()
        .find(|(i, c)| i > &index && !c.is_whitespace());

    if let Some((index, _)) = index {
        let position = get_position_on_word_end(content, index, is_upper);
        if let Ok(mut position) = position {
            if let CursorPosition::Absolute { current, expanded } = &mut position {
                *current = content_len - *current - 1;
                *expanded = content_len - *expanded - 1;
            }
            cursor.horizontal_index = position;
        }
    } else {
        let new_line_cursor = match get_cursor_on_word_previous_line(cursor, &model.lines) {
            Ok(crsr) => crsr,
            Err(_) => return,
        };

        let current = match model.lines.get(new_line_cursor.vertical_index) {
            Some(line) => line,
            None => return,
        };

        let content = current
            .content
            .to_stripped_string()
            .chars()
            .rev()
            .collect::<Vec<_>>();

        let content_len = content.len();
        let new_line_index =
            match cursor::get_horizontal_index(&new_line_cursor.horizontal_index, current) {
                Some(index) => content_len - index - 1,
                None => return,
            };

        let position = get_position_on_word_end(content, new_line_index, is_upper);
        if let Ok(mut position) = position {
            if let CursorPosition::Absolute { current, expanded } = &mut position {
                *current = content_len - *current - 1;
                *expanded = content_len - *expanded - 1;
            }
            cursor.vertical_index = new_line_cursor.vertical_index;
            cursor.horizontal_index = position;
        }
    }
}

fn get_position_on_word_end(
    content: Vec<char>,
    index: usize,
    is_upper: bool,
) -> Result<CursorPosition, ()> {
    let char = content.get(index).ok_or(())?;
    let is_alphanumeric = char.is_alphanumeric() || char == &'_';

    let predicate = match (is_upper, is_alphanumeric) {
        (true, _) => |c: &char| c.is_whitespace(),
        (false, true) => |c: &char| c != &'_' && !c.is_alphanumeric(),
        (false, false) => |c: &char| c == &'_' || c.is_alphanumeric() || c.is_whitespace(),
    };

    let next = content
        .iter()
        .enumerate()
        .position(|(i, c)| i > index && predicate(c))
        .map(|i| if i == 0 { 0 } else { i - 1 });

    if let Some(next_index) = next {
        Ok(CursorPosition::Absolute {
            current: next_index,
            expanded: next_index,
        })
    } else {
        Ok(CursorPosition::Absolute {
            current: content.len() - 1,
            expanded: content.len() - 1,
        })
    }
}

fn get_cursor_on_word_next_line(cursor: &Cursor, lines: &[BufferLine]) -> Result<Cursor, ()> {
    let mut result = cursor.clone();
    let max_index = lines.len() - 1;
    if cursor.vertical_index >= max_index {
        result.vertical_index = max_index;
        return Ok(result);
    }

    result.vertical_index += 1;
    result.horizontal_index = CursorPosition::Absolute {
        current: 0,
        expanded: 0,
    };

    let current = match lines.get(result.vertical_index) {
        Some(line) => line,
        None => return Err(()),
    };

    let index = match cursor::get_horizontal_index(&result.horizontal_index, current) {
        Some(index) => index,
        None => return Err(()),
    };

    let content = current
        .content
        .to_stripped_string()
        .chars()
        .skip(index)
        .collect::<Vec<_>>();

    if content.first().is_some_and(|c| c.is_whitespace()) {
        let next = content
            .iter()
            .position(|c| !c.is_whitespace())
            .map(|i| index + i);

        if let Some(next_index) = next {
            result.horizontal_index = CursorPosition::Absolute {
                current: next_index,
                expanded: next_index,
            };
        } else {
            return Err(());
        };
    }

    Ok(result)
}

fn get_cursor_on_word_previous_line(cursor: &Cursor, lines: &[BufferLine]) -> Result<Cursor, ()> {
    let mut result = cursor.clone();
    if cursor.vertical_index == 0 {
        return Ok(result);
    }

    result.vertical_index -= 1;
    let current = match lines.get(result.vertical_index) {
        Some(line) => line,
        None => return Err(()),
    };

    let content_len = current.content.count_chars();
    result.horizontal_index = CursorPosition::Absolute {
        current: content_len - 1,
        expanded: content_len - 1,
    };

    let content = current
        .content
        .to_stripped_string()
        .chars()
        .rev()
        .collect::<Vec<_>>();

    if content.first().is_some_and(|c| c.is_whitespace()) {
        let next = content.iter().position(|c| !c.is_whitespace());

        if let Some(next_index) = next {
            result.horizontal_index = CursorPosition::Absolute {
                current: content_len - next_index - 1,
                expanded: content_len - next_index - 1,
            };
        } else {
            return Err(());
        };
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use crate::model::{Buffer, BufferLine, Cursor, CursorPosition};

    #[test]
    fn move_cursor_to_word_end_backward_starting_on_line_start() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello worldz"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 1;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 0,
            expanded: 0,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "_ello world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_backward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello world_");
    }

    #[test]
    fn move_cursor_to_word_end_backward_starting_on_word() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello worldz"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 6,
            expanded: 6,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello _orldz");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_backward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hell_ worldz");
    }

    #[test]
    fn move_cursor_to_word_start_starting_on_word() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 0,
            expanded: 0,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "_ello world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello _orld");
    }

    #[test]
    fn move_cursor_to_word_start_starting_on_word_middle() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 1,
            expanded: 1,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "h_llo world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello _orld");
    }

    #[test]
    fn move_cursor_to_word_start_starting_on_last_word() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 7,
            expanded: 7,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello w_rld");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 1);

        assert_cursor_position_eq(&buffer.lines, &cursor, "_ello world");
    }

    #[test]
    fn move_cursor_to_word_start_starting_on_last_word_with_whitespace() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world  "),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 7,
            expanded: 7,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello w_rld  ");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 1);

        assert_cursor_position_eq(&buffer.lines, &cursor, "_ello world");
    }

    #[test]
    fn move_cursor_to_word_start_changing_alphanumeric() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello!@#$!@#$"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 1,
            expanded: 1,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "h_llo!@#$!@#$");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello_@#$!@#$");
    }

    #[test]
    fn move_cursor_to_word_end_starting_on_word() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 1,
            expanded: 1,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "h_llo world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hell_ world");
    }

    #[test]
    fn move_cursor_to_word_end_starting_on_whitespace() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("    hello world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 1,
            expanded: 1,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, " _  hello world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "    hell_ world");
    }

    #[test]
    fn move_cursor_to_word_end_starting_on_wordend() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world "),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 4,
            expanded: 4,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "hell_ world ");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello worl_ ");
    }

    #[test]
    fn move_cursor_to_word_end_starting_on_wordend_at_lineend() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 10,
            expanded: 10,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello worl_");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 1);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hell_ world");
    }

    #[test]
    fn move_cursor_to_word_end_starting_on_wordend_at_lineend_with_whitespaces() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world   "),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 10,
            expanded: 10,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello worl_   ");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 1);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hell_ world");
    }

    #[test]
    fn move_cursor_to_word_end_jump_to_wordend_on_lineend() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("!@#- world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 3,
            expanded: 3,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "!@#_ world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "!@#- worl_");
    }

    #[test]
    fn move_cursor_to_word_end_two_words_without_whitespace() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hell#-world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 5,
            expanded: 5,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "hell#_world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_forward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hell#-worl_");
    }

    #[test]
    fn move_cursor_to_word_end_within_upper_word() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hell#-world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 0,
            expanded: 0,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "_ell#-world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_end_forward(&mut buffer, true);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hell#-worl_");
    }

    #[test]
    fn move_cursor_to_word_start_backward_starting_on_line_start() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello worldz"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 1;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 0,
            expanded: 0,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "_ello world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_backward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello _orldz");
    }

    #[test]
    fn move_cursor_to_word_start_backward_starting_on_word() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello worldz"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 6,
            expanded: 6,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello _orldz");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_backward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "_ello worldz");
    }

    #[test]
    fn move_cursor_to_word_start_backward_starting_on_whitespace() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world"),
            BufferLine::from("    hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 1;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 1,
            expanded: 1,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, " _  hello world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_backward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello _orld");
    }

    #[test]
    fn move_cursor_to_word_start_backward_starting_on_word_middle() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 1,
            expanded: 1,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "h_llo world");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_backward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "_ello world");
    }

    #[test]
    fn move_cursor_to_word_start_backward_starting_on_last_word() {
        let mut buffer = Buffer::default();
        buffer.lines = vec![
            BufferLine::from("hello world"),
            BufferLine::from("hello world"),
        ];

        let mut cursor = Cursor::default();
        cursor.vertical_index = 0;
        cursor.horizontal_index = CursorPosition::Absolute {
            current: 7,
            expanded: 7,
        };

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello w_rld");

        buffer.cursor = Some(cursor);

        super::move_cursor_to_word_start_backward(&mut buffer, false);

        let cursor = buffer.cursor.unwrap();
        assert_eq!(cursor.vertical_index, 0);

        assert_cursor_position_eq(&buffer.lines, &cursor, "hello _orld");
    }

    fn assert_cursor_position_eq(lines: &Vec<BufferLine>, cursor: &Cursor, expected: &str) {
        if let CursorPosition::Absolute {
            current: current_index,
            expanded: _,
        } = &cursor.horizontal_index
        {
            let position = *current_index;
            let mut current = lines
                .get(cursor.vertical_index)
                .unwrap()
                .content
                .to_string();
            current.replace_range(position..position + 1, "_");

            assert_eq!(current, expected);
        } else {
            panic!("Expected CursorPosition::Absolute");
        }
    }
}
