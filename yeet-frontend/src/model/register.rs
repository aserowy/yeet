use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Register {
    pub command: Option<String>,
    pub content: HashMap<char, String>,
    pub dot: Option<String>,
    pub find: Option<String>,
    pub r#macro: Option<String>,
    pub searched: Option<String>,
    pub scopes: HashMap<RegisterScope, String>,
}

impl Register {
    pub fn get(&self, register: &char) -> Option<String> {
        match register {
            '@' => self.r#macro.clone(),
            '.' => self.dot.clone(),
            ';' => self.find.clone(),
            ':' => self.command.clone(),
            '/' => self.searched.clone(),
            char => self.content.get(char).cloned(),
        }
    }

    pub fn resolve_macro(&self) -> Option<&RegisterScope> {
        self.scopes
            .keys()
            .find(|scope| matches!(scope, RegisterScope::Macro(_)))
    }

    pub fn print(&self) -> Vec<String> {
        let mut contents = vec![":reg".to_string(), "Name Content".to_string()];

        for (key, content) in self.content.iter() {
            contents.push(print_content(key, content));
        }

        if let Some(r#macro) = &self.r#macro {
            contents.push(print_content(&'@', r#macro));
        }
        if let Some(dot) = &self.dot {
            contents.push(print_content(&'.', dot));
        }
        if let Some(command) = &self.command {
            contents.push(print_content(&':', command));
        }
        if let Some(find) = &self.find {
            contents.push(print_content(&';', find));
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

#[derive(Clone, Debug, Eq)]
pub enum RegisterScope {
    Dot,
    Find,
    Macro(char),
    Search,
}

impl Hash for RegisterScope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RegisterScope::Dot => state.write_u8(2),
            RegisterScope::Find => state.write_u8(3),
            RegisterScope::Macro(_) => state.write_u8(4),
            RegisterScope::Search => state.write_u8(5),
        }
    }
}

impl PartialEq for RegisterScope {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (RegisterScope::Dot, RegisterScope::Dot)
                | (RegisterScope::Find, RegisterScope::Find)
                | (RegisterScope::Macro(_), RegisterScope::Macro(_))
                | (RegisterScope::Search, RegisterScope::Search)
        )
    }
}
