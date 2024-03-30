#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Register {
    pub command: Option<String>,
    pub current: RegisterType,
    pub dot: Option<String>,
    pub searched: Option<String>,
    pub trashed: Vec<String>,
    pub yanked: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum RegisterType {
    _Custom(String),
    _Trash,
    #[default]
    Yank,
}

impl Register {
    pub fn print(&self) -> Vec<String> {
        let mut contents = vec![":reg".to_string(), "Name Content".to_string()];
        if let Some(yanked) = &self.yanked {
            contents.push(print_content(&'0', yanked));
        }
        if !self.trashed.is_empty() {
            contents.extend(self.trashed.iter().map(|t| format!("\"{}", t)));
        }
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
