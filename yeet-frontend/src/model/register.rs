use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use yeet_buffer::model::SearchDirection;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Register {
    pub command: Option<String>,
    pub content: HashMap<char, String>,
    pub dot: Option<String>,
    pub find: Option<String>,
    pub last_macro: Option<String>,
    pub searched: Option<(SearchDirection, String)>,
    pub scopes: HashMap<RegisterScope, String>,
}

// TODO: move all methods to update mod as pure functions
impl Register {
    pub fn get(&self, register: &char) -> Option<String> {
        match register {
            '@' => self.last_macro.clone(),
            '.' => self.dot.clone(),
            ';' => self.find.clone(),
            ':' => self.command.clone(),
            '/' => self.searched.as_ref().map(|sd| sd.1.clone()),
            char => self.content.get(char).cloned(),
        }
    }

    pub fn get_search_direction(&self) -> Option<&SearchDirection> {
        self.searched.as_ref().map(|sd| &sd.0)
    }

    pub fn resolve_macro(&self) -> Option<&RegisterScope> {
        self.scopes
            .keys()
            .find(|scope| matches!(scope, RegisterScope::Macro(_)))
    }
}

#[derive(Clone, Debug, Eq)]
pub enum RegisterScope {
    Dot,
    Find,
    Macro(char),
}

impl Hash for RegisterScope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RegisterScope::Dot => state.write_u8(2),
            RegisterScope::Find => state.write_u8(3),
            RegisterScope::Macro(_) => state.write_u8(4),
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
        )
    }
}
