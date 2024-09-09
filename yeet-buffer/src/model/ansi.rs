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
                content.push(c);
            } else if is_ansi && c == 'm' {
                is_ansi = false;
                content.push(c);
            } else if is_ansi {
                content.push(c);
            } else if !is_ansi {
                current_count += 1;
                if current_count > count {
                    content.push(c);
                }
            }
        }
        return Self::new(&content);
    }

    pub fn take_chars(&self, count: usize) -> Self {
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
        self.content = self.take_chars(index).skip_chars(count).content;
    }

    pub fn get_index_for_char(&self, count: usize) -> Option<usize> {
        let mut current_count = 0;
        let mut is_ansi = false;
        for (i, c) in self.content.char_indices() {
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
                current_count += 1;
            }
            if current_count == count {
                break;
            }
        }
        result
    }

    pub fn is_empty(&self) -> bool {
        self.count_chars() == 0
    }
}
