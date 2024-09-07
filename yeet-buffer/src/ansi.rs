pub fn get_char_count(s: &str) -> usize {
    let mut count = 0;
    let mut is_ansi = false;
    for c in s.chars() {
        if c == '\x1b' {
            is_ansi = true;
        } else if is_ansi && c == 'm' {
            is_ansi = false;
        } else if !is_ansi {
            count += 1;
        }
    }
    return count;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_char_count_with_no_ansi_codes() {
        let input = "Hello, World!";
        let expected = 13;
        assert_eq!(get_char_count(input), expected);
    }

    #[test]
    fn get_char_count_with_ansi_codes() {
        let input = "\x1b[31mHello\x1b[0m, \x1b[32mWorld\x1b[0m!";
        let expected = 13;
        assert_eq!(get_char_count(input), expected);
    }

    #[test]
    fn get_char_count_empty_string() {
        let input = "";
        let expected = 0;
        assert_eq!(get_char_count(input), expected);
    }

    #[test]
    fn get_char_count_only_ansi_codes() {
        let input = "\x1b[31m\x1b[0m";
        let expected = 0;
        assert_eq!(get_char_count(input), expected);
    }

    #[test]
    fn get_char_count_mixed_content() {
        let input = "Hello\x1b[31m, \x1b[0mWorld!";
        let expected = 13;
        assert_eq!(get_char_count(input), expected);
    }
}
