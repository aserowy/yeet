#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Ansi {
    content: String,
}

impl ToString for Ansi {
    fn to_string(&self) -> String {
        self.content.clone()
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

    pub fn skip_chars(&self, count: usize) -> Self {
        let mut current_count = 0;
        let mut is_ansi = false;
        let mut content = String::new();
        for c in self.content.chars() {
            if c == '\x1b' {
                is_ansi = true;
            } else if is_ansi && c == 'm' {
                is_ansi = false;
            } else if !is_ansi {
                current_count += 1;
            }
            if current_count > count {
                content.push(c);
            }
        }
        return Self::new(&content);
    }

    pub fn take_chars(&self, count: usize) -> Self {
        if count == 0 {
            return Self::new("");
        }
        let mut current_count = 0;
        let mut is_ansi = false;
        let mut content = String::new();
        for c in self.content.chars() {
            if c == '\x1b' {
                is_ansi = true;
                content.push(c);
            } else if is_ansi && c == 'm' {
                is_ansi = false;
                content.push(c);
            } else if is_ansi {
                content.push(c);
            } else if !is_ansi {
                current_count += 1;
                if current_count <= count {
                    content.push(c);
                }
            }
        }
        return Self::new(&content);
    }

    pub fn join(&mut self, other: &Self) -> Self {
        Ansi::new(&format!("{}{}", self.content, other.content))
    }

    pub fn append(&mut self, s: &str) {
        self.content.push_str(s);
    }

    pub fn insert(&mut self, index: usize, s: &str) {
        self.content = format!(
            "{}{}{}",
            self.take_chars(index).to_string(),
            s,
            self.skip_chars(index).to_string()
        );
    }

    pub fn prepend(&mut self, s: &str) {
        self.content.insert_str(0, s);
    }

    pub fn remove(&mut self, index: usize, count: usize) {
        self.content = self
            .take_chars(index)
            .join(&self.skip_chars(index + count))
            .content;
    }

    pub fn get_ansi_escape_sequences_till_char(&self, count: usize) -> String {
        if count == 0 {
            return String::new();
        }

        let mut current_count = 0;
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
                if current_count == count {
                    break;
                }
                current_count += 1;
            }
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.count_chars() == 0
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
        let skipped = ansi.skip_chars(7);
        assert_eq!(skipped.content, "\x1b[31mworld\x1b[0m!");
    }

    #[test]
    fn test_take_chars() {
        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        let taken = ansi.take_chars(5);
        assert_eq!(taken.content, "Hello\u{1b}[31m\u{1b}[0m");

        let ansi = Ansi::new("\x1b[12mHello, \x1b[31mworld\x1b[0m!");
        let taken = ansi.take_chars(0);
        assert_eq!(taken.content, "");
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
        assert_eq!(ansi.content, "Hello, \x1b[31m\x1b[0m\x1b[31m\x1b[0m!");
    }

    #[test]
    fn test_get_ansi_escape_sequences_till_char() {
        let ansi = Ansi::new("Hello, \x1b[31mworld\x1b[0m!");
        assert_eq!(ansi.get_ansi_escape_sequences_till_char(7), "\x1b[31m");
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
