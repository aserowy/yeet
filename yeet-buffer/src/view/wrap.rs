use crate::model::ansi::Ansi;

pub struct WrapSegment {
    pub content: Ansi,
    pub is_first: bool,
    pub char_offset: usize,
}

pub fn wrap_line(content: &Ansi, content_width: usize) -> Vec<WrapSegment> {
    if content_width == 0 {
        return vec![WrapSegment {
            content: Ansi::new(""),
            is_first: true,
            char_offset: 0,
        }];
    }

    let total_chars = content.count_chars();
    if total_chars <= content_width {
        return vec![WrapSegment {
            content: content.clone(),
            is_first: true,
            char_offset: 0,
        }];
    }

    let stripped = content.to_stripped_string();
    let mut segments = Vec::new();
    let mut offset = 0;
    let mut is_first = true;

    while offset < total_chars {
        let remaining = total_chars - offset;
        if remaining <= content_width {
            let mut seg = content.skip_chars(offset);
            if !is_first {
                let style_prefix = content.get_ansi_escape_sequences_till_char(offset);
                seg.prepend(&style_prefix);
            }
            segments.push(WrapSegment {
                content: seg,
                is_first,
                char_offset: offset,
            });
            break;
        }

        let end = offset + content_width;
        let slice = &stripped[char_index(&stripped, offset)..char_index(&stripped, end)];
        let break_at = match slice.rfind(' ') {
            Some(space_pos) => {
                let chars_before_space = stripped
                    [char_index(&stripped, offset)..char_index(&stripped, offset) + space_pos]
                    .chars()
                    .count();
                if chars_before_space == 0 {
                    content_width
                } else {
                    chars_before_space
                }
            }
            None => content_width,
        };

        let mut segment_content = content.skip_chars(offset).take_chars(break_at);
        if !is_first {
            let style_prefix = content.get_ansi_escape_sequences_till_char(offset);
            segment_content.prepend(&style_prefix);
        }
        segments.push(WrapSegment {
            content: segment_content,
            is_first,
            char_offset: offset,
        });

        offset += break_at;
        if offset < total_chars {
            let next_char = stripped.chars().nth(offset);
            if next_char == Some(' ') {
                offset += 1;
            }
        }

        is_first = false;
    }

    if segments.is_empty() {
        segments.push(WrapSegment {
            content: Ansi::new(""),
            is_first: true,
            char_offset: 0,
        });
    }

    segments
}

fn char_index(s: &str, char_pos: usize) -> usize {
    s.char_indices()
        .nth(char_pos)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

pub fn visual_line_count(content: &Ansi, content_width: usize) -> usize {
    if content_width == 0 {
        return 1;
    }
    let total_chars = content.count_chars();
    if total_chars <= content_width {
        return 1;
    }
    wrap_line(content, content_width).len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_shorter_than_width_single_segment() {
        let content = Ansi::new("hello");
        let segments = wrap_line(&content, 10);
        assert_eq!(segments.len(), 1);
        assert!(segments[0].is_first);
        assert_eq!(segments[0].char_offset, 0);
        assert_eq!(segments[0].content.to_stripped_string(), "hello");
    }

    #[test]
    fn line_equal_to_width_single_segment() {
        let content = Ansi::new("hello worl");
        let segments = wrap_line(&content, 10);
        assert_eq!(segments.len(), 1);
    }

    #[test]
    fn line_breaks_at_space() {
        let content = Ansi::new("hello world foo");
        let segments = wrap_line(&content, 10);
        assert_eq!(segments.len(), 2);
        assert!(segments[0].is_first);
        assert_eq!(segments[0].content.to_stripped_string(), "hello");
        assert!(!segments[1].is_first);
        assert_eq!(segments[1].content.to_stripped_string(), "world foo");
    }

    #[test]
    fn word_longer_than_width_breaks_by_char() {
        let content = Ansi::new("abcdefghijklmno");
        let segments = wrap_line(&content, 10);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].content.to_stripped_string(), "abcdefghij");
        assert_eq!(segments[1].content.to_stripped_string(), "klmno");
    }

    #[test]
    fn multiple_wraps() {
        let content = Ansi::new("aa bb cc dd ee ff");
        let segments = wrap_line(&content, 6);
        assert!(segments.len() >= 3);
        assert!(segments[0].is_first);
        assert!(!segments[1].is_first);
        assert!(!segments[2].is_first);
    }

    #[test]
    fn line_with_ansi_codes_wraps_correctly() {
        let content = Ansi::new("\x1b[31mhello world\x1b[0m foo");
        let segments = wrap_line(&content, 10);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].content.to_stripped_string(), "hello");
        assert_eq!(segments[1].content.to_stripped_string(), "world foo");
    }

    #[test]
    fn empty_line_returns_one_segment() {
        let content = Ansi::new("");
        let segments = wrap_line(&content, 10);
        assert_eq!(segments.len(), 1);
        assert!(segments[0].is_first);
        assert_eq!(segments[0].content.to_stripped_string(), "");
    }

    #[test]
    fn char_offsets_are_correct() {
        let content = Ansi::new("hello world foo");
        let segments = wrap_line(&content, 10);
        assert_eq!(segments[0].char_offset, 0);
        assert_eq!(segments[1].char_offset, 6);
    }

    #[test]
    fn visual_line_count_short_line() {
        let content = Ansi::new("hello");
        assert_eq!(visual_line_count(&content, 10), 1);
    }

    #[test]
    fn visual_line_count_wrapped_line() {
        let content = Ansi::new("hello world foo");
        assert_eq!(visual_line_count(&content, 10), 2);
    }

    #[test]
    fn continuation_segment_carries_ansi_style() {
        let content = Ansi::new("\x1b[31mhello world\x1b[0m foo");
        let segments = wrap_line(&content, 10);
        assert_eq!(segments.len(), 2);
        let raw = segments[1].content.to_string();
        assert!(
            raw.contains("\x1b[31m"),
            "continuation segment should start with the red ANSI code, got: {:?}",
            raw,
        );
    }

    #[test]
    fn first_segment_has_no_extra_prefix() {
        let content = Ansi::new("\x1b[31mhello world\x1b[0m foo");
        let segments = wrap_line(&content, 10);
        let first_raw = segments[0].content.to_string();
        assert!(
            first_raw.starts_with("\x1b[31m"),
            "first segment should start with original ANSI code"
        );
        assert!(
            !first_raw.starts_with("\x1b[31m\x1b[31m"),
            "first segment should not have duplicated ANSI prefix"
        );
    }
}
