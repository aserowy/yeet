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
    count
}

pub fn get_index_for_char(s: &str, count: usize) -> Option<usize> {
    let mut current_count = 0;
    let mut is_ansi = false;
    for (i, c) in s.char_indices() {
        if c == '\x1b' {
            is_ansi = true;
        } else if is_ansi && c == 'm' {
            is_ansi = false;
        } else if !is_ansi {
            current_count += 1;
        }
        if current_count == count {
            return Some(i);
        }
    }
    None
}

pub fn get_ansi_escape_sequences_till_char_count(s: &str, count: usize) -> String {
    if count == 0 {
        return String::new();
    }

    let mut current_count = 0;
    let mut is_ansi = false;
    let mut result = String::new();
    for c in s.chars() {
        if c == '\x1b' {
            is_ansi = true;
            result.push(c);
        } else if is_ansi && c == 'm' {
            is_ansi = false;
            result.push(c);
        } else if is_ansi {
            result.push(c);
        } else if !is_ansi {
            current_count += 1;
        }
        if current_count == count {
            break;
        }
    }
    result
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

    #[test]
    fn get_index_for_char_position_normal() {
        let s = "hello";
        let char_count = 2;
        let index = get_index_for_char(s, char_count);
        assert_eq!(index, Some(1));
    }

    #[test]
    fn get_index_for_char_count_start() {
        let s = "hello";
        let char_count = 0;
        let index = get_index_for_char(s, char_count);
        assert_eq!(index, None);
    }

    #[test]
    fn get_index_for_char_count_end() {
        let s = "hello";
        let char_count = 4;
        let index = get_index_for_char(s, char_count);
        assert_eq!(index, Some(3));
    }

    #[test]
    fn get_index_for_char_count_out_of_bounds() {
        let s = "hello";
        let char_count = 10;
        let index = get_index_for_char(s, char_count);
        assert_eq!(index, None);
    }

    #[test]
    fn get_index_for_char_count_with_ansi_escape_code() {
        let s = "\x1b[31mhello\x1b[0m";
        let char_count = 1;
        let index = get_index_for_char(s, char_count);
        assert_eq!(index, Some(5));
    }

    #[test]
    fn get_index_for_char_count_with_multiple_ansi_escape_codes() {
        let s = "\x1b[31mhe\x1b[32mllo\x1b[0m";
        let char_count = 3;
        let index = get_index_for_char(s, char_count);
        assert_eq!(index, Some(12));
    }

    #[test]
    fn get_ansi_escape_sequences_till_char_count_normal() {
        let s = "hello";
        let count = 2;
        let result = get_ansi_escape_sequences_till_char_count(s, count);
        assert_eq!(result, "");
    }

    #[test]
    fn get_ansi_escape_sequences_till_char_count_start() {
        let s = "hello";
        let count = 0;
        let result = get_ansi_escape_sequences_till_char_count(s, count);
        assert_eq!(result, "");
    }

    #[test]
    fn get_ansi_escape_sequences_till_char_count_end() {
        let s = "hello";
        let count = 5;
        let result = get_ansi_escape_sequences_till_char_count(s, count);
        assert_eq!(result, "");
    }

    #[test]
    fn get_ansi_escape_sequences_till_char_count_out_of_bounds() {
        let s = "hello";
        let count = 10;
        let result = get_ansi_escape_sequences_till_char_count(s, count);
        assert_eq!(result, "");
    }

    #[test]
    fn get_ansi_escape_sequences_till_char_count_with_ansi_escape_codes_at_start() {
        let s = "\x1b[31mhello\x1b[0m";
        let count = 1;
        let result = get_ansi_escape_sequences_till_char_count(s, count);
        assert_eq!(result, "\x1b[31m");
    }

    #[test]
    fn get_ansi_escape_sequences_till_char_count_with_ansi_escape_codes() {
        let s = "\x1b[31mhello\x1b[0m";
        let count = 0;
        let result = get_ansi_escape_sequences_till_char_count(s, count);
        assert_eq!(result, "");
    }

    #[test]
    fn get_ansi_escape_sequences_till_char_count_with_ansi_escape_codes_out_of_bounds() {
        let s = "\x1b[31mhello\x1b[0m";
        let count = 10;
        let result = get_ansi_escape_sequences_till_char_count(s, count);
        assert_eq!(result, "\x1b[31m\x1b[0m");
    }
}
