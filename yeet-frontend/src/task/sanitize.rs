pub fn strip_non_sgr_escape_sequences(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\x1b' {
            result.push(c);
            continue;
        }

        if chars.peek() != Some(&'[') {
            result.push(c);
            continue;
        }

        let mut sequence = String::from("\x1b[");
        chars.next();

        loop {
            match chars.next() {
                Some(sc) if sc.is_ascii_alphabetic() => {
                    if sc == 'm' {
                        sequence.push(sc);
                        result.push_str(&sequence);
                    }
                    break;
                }
                Some(sc) => {
                    sequence.push(sc);
                }
                None => break,
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_sgr_sequences() {
        let input = "\x1b[38;2;255;100;50mhello\x1b[0m";
        assert_eq!(strip_non_sgr_escape_sequences(input), input);
    }

    #[test]
    fn strips_cursor_movement_sequences() {
        let input = "\x1b[2Chello";
        assert_eq!(strip_non_sgr_escape_sequences(input), "hello");
    }

    #[test]
    fn strips_cursor_position_sequences() {
        let input = "\x1b[1;1Hhello";
        assert_eq!(strip_non_sgr_escape_sequences(input), "hello");
    }

    #[test]
    fn strips_erase_sequences() {
        let input = "hello\x1b[2J\x1b[Kworld";
        assert_eq!(strip_non_sgr_escape_sequences(input), "helloworld");
    }

    #[test]
    fn mixed_sgr_and_non_sgr() {
        let input = "\x1b[2C\x1b[31mhello\x1b[0m\x1b[1A";
        assert_eq!(
            strip_non_sgr_escape_sequences(input),
            "\x1b[31mhello\x1b[0m"
        );
    }

    #[test]
    fn plain_text_unchanged() {
        let input = "hello world";
        assert_eq!(strip_non_sgr_escape_sequences(input), input);
    }

    #[test]
    fn empty_string_unchanged() {
        assert_eq!(strip_non_sgr_escape_sequences(""), "");
    }

    #[test]
    fn preserves_non_csi_escape_sequences() {
        let input = "\x1b]hello\x1b[31mworld\x1b[0m";
        assert_eq!(
            strip_non_sgr_escape_sequences(input),
            "\x1b]hello\x1b[31mworld\x1b[0m"
        );
    }
}
