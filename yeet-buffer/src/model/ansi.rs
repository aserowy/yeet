use std::fmt::Display;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ansi {
    content: String,
}

impl Display for Ansi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.content.fmt(f)
    }
}

impl Ansi {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }

    pub fn to_stripped_string(&self) -> String {
        let mut is_ansi = false;
        let mut result = String::new();
        for c in self.content.chars() {
            if c == '\x1b' {
                is_ansi = true;
            } else if is_ansi && c == 'm' {
                is_ansi = false;
            } else if !is_ansi {
                result.push(c);
            }
        }
        result
    }

    pub fn count_chars(&self) -> usize {
        let mut count = 0;
        let mut is_ansi = false;
        for c in self.content.chars() {
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

    pub fn skip_chars(&self, position: usize) -> Self {
        if position == 0 {
            return self.clone();
        }
        let index = self.position_to_index(position);
        match self.content.get(index..) {
            Some(content) => Self::new(content),
            None => Ansi::new(""),
        }
    }

    pub fn take_chars(&self, position: usize) -> Self {
        if position == 0 {
            return Ansi::new("");
        }
        let index = self.position_to_index(position);
        match self.content.get(..index) {
            Some(content) => Self::new(content),
            None => self.clone(),
        }
    }

    pub fn join(&mut self, other: &Self) -> Self {
        Ansi::new(&format!("{}{}", self.content, other.content))
    }

    pub fn append(&mut self, s: &str) {
        self.content.push_str(s);
    }

    pub fn insert(&mut self, position: usize, s: &str) {
        let index = self.position_to_index(position);
        self.content.insert_str(index, s);
    }

    pub fn prepend(&mut self, s: &str) {
        self.content.insert_str(0, s);
    }

    pub fn remove(&mut self, position: usize, size: usize) {
        let index_start = self.position_to_index(position);
        let index_end = self.position_to_index(position + size);
        for _ in 0..(index_end - index_start) {
            self.content.remove(index_start);
        }
    }

    pub fn get_ansi_escape_sequences_till_char(&self, position: usize) -> String {
        let mut current_position = 0;
        let mut is_ansi = false;
        let mut result = String::new();
        for c in self.content.chars() {
            if c == '\x1b' {
                is_ansi = true;
                result.push(c);
            } else if is_ansi && c == 'm' {
                is_ansi = false;
                result.push(c);
            } else if is_ansi {
                result.push(c);
            } else if !is_ansi {
                current_position += 1;
                if current_position > position {
                    break;
                }
            }
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.count_chars() == 0
    }

    fn position_to_index(&self, position: usize) -> usize {
        let mut current_position = 0;
        let mut is_ansi = false;
        for (i, c) in self.content.chars().enumerate() {
            if c == '\x1b' {
                is_ansi = true;
            } else if is_ansi && c == 'm' {
                is_ansi = false;
            } else if !is_ansi {
                if current_position == position {
                    return i;
                }
                current_position += 1;
            }
        }
        return self.content.chars().count();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        assert_eq!(ansi.content, "Hello, \x1b[31mworld\x1b[0m!");
    }

    #[test]
    fn test_to_stripped_string() {
        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        assert_eq!(ansi.to_stripped_string(), "Hello, world!");
    }

    #[test]
    fn test_count_chars() {
        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        assert_eq!(ansi.count_chars(), 13);
    }

    #[test]
    fn test_skip_chars() {
        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        let skipped = ansi.skip_chars(5);
        assert_eq!(skipped.content, ", \x1b[31mworld\x1b[0m!");

        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        let skipped = ansi.skip_chars(7);
        assert_eq!(skipped.content, "world\x1b[0m!");

        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        let skipped = ansi.skip_chars(0);
        assert_eq!(skipped.content, "Hello, \x1b[31mworld\x1b[0m!");

        let ansi = Ansi::new("\x1b[31mHello, \x1b[31mworld\x1b[0m!");
        let skipped = ansi.skip_chars(0);
        assert_eq!(skipped.content, "\x1b[31mHello, \x1b[31mworld\x1b[0m!");

        let ansi = Ansi::new("Hello");
        let skipped = ansi.skip_chars(5);
        assert_eq!(skipped.content, "");
    }

    #[test]
    fn test_take_chars() {
        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        let taken = ansi.take_chars(5);
        assert_eq!(taken.content, "Hello");

        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        let taken = ansi.take_chars(8);
        assert_eq!(taken.content, "Hello, \x1b[31mw");

        let ansi = Ansi::new("\x1b[12mHello, \x1b[31mworld\x1b[0m!");
        let taken = ansi.take_chars(0);
        assert_eq!(taken.content, "");

        let ansi = Ansi::new("Hello");
        let skipped = ansi.take_chars(5);
        assert_eq!(skipped.content, "Hello");
    }

    #[test]
    fn test_join() {
        let mut ansi1 = Ansi::new("Hello, ");
        let ansi2 = Ansi::new("\x1b[31mworld\x1b[0m!");
        let joined = ansi1.join(&ansi2);
        assert_eq!(joined.content, "Hello, \x1b[31mworld\x1b[0m!");
    }

    #[test]
    fn test_append() {
        let mut ansi = Ansi::new("Hello");
        ansi.append(", \x1b[31mworld\x1b[0m!");
        assert_eq!(ansi.content, "Hello, \x1b[31mworld\x1b[0m!");
    }

    #[test]
    fn test_insert() {
        let mut ansi = Ansi::new("Hello, world!");
        ansi.insert(7, "asdf\x1b[31m");
        assert_eq!(ansi.content, "Hello, asdf\x1b[31mworld!");

        ansi.insert(0, "\x1b[31m");
        assert_eq!(ansi.content, "\x1b[31mHello, asdf\x1b[31mworld!");

        ansi.insert(0, "\x1b[1m");
        assert_eq!(ansi.content, "\x1b[31m\x1b[1mHello, asdf\x1b[31mworld!");

        let mut ansi = Ansi::new("");
        ansi.insert(0, "1");
        ansi.insert(1, "2");
        ansi.insert(2, "3");
        assert_eq!(ansi.content, "123");
    }

    #[test]
    fn test_prepend() {
        let mut ansi = Ansi::new("world!");
        ansi.prepend("Hello, \x1b[31m");
        assert_eq!(ansi.content, "Hello, \x1b[31mworld!");
    }

    #[test]
    fn test_remove() {
        let mut ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        ansi.remove(7, 5);
        assert_eq!(ansi.content, "Hello, \x1b[31m!");
    }

    #[test]
    fn test_get_ansi_escape_sequences_till_char() {
        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        assert_eq!(ansi.get_ansi_escape_sequences_till_char(7), "\x1b[31m");

        let ansi = Ansi::new("\x1b[31mworld\x1b[0m!");
        assert_eq!(ansi.get_ansi_escape_sequences_till_char(0), "\x1b[31m");
    }

    #[test]
    fn test_is_empty() {
        let ansi = Ansi::new("");
        assert!(ansi.is_empty());

        let ansi = Ansi::new("\x1b[31m\x1b[0m");
        assert!(ansi.is_empty());

        let ansi = Ansi::new("Hello");
        assert!(!ansi.is_empty());
    }
}
