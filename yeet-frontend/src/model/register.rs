#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Register {
    pub command: Option<String>,
    pub dot: Option<String>,
    pub searched: Option<String>,
    pub scope: Option<RegisterScope>,
}

impl Register {
    pub fn print(&self) -> Vec<String> {
        let mut contents = vec![":reg".to_string(), "Name Content".to_string()];
        if let Some(dot) = &self.dot {
            contents.push(print_content(&'.', dot));
        }
        if let Some(command) = &self.command {
            contents.push(print_content(&':', command));
        }
        if let Some(searched) = &self.searched {
            contents.push(print_content(&'/', searched));
        }
        contents
    }
}

fn print_content(prefix: &char, content: &str) -> String {
    format!("\"{:<3} {}", prefix, content)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisterScope {
    _Command,
    Dot,
    _Find,
    _Macro(char),
    _Search,
}
