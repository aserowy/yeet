use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use arboard::Clipboard;
use yeet_buffer::model::SearchDirection;

pub struct Register {
    pub clipboard: Option<Clipboard>,
    pub command: Option<String>,
    pub content: HashMap<char, String>,
    pub dot: Option<String>,
    pub last_macro: Option<String>,
    pub searched: Option<(SearchDirection, String)>,
    pub scopes: HashMap<RegisterScope, String>,
}

impl Default for Register {
    fn default() -> Self {
        Self {
            clipboard: Clipboard::new().ok(),
            command: None,
            content: Default::default(),
            dot: None,
            last_macro: None,
            searched: None,
            scopes: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Eq)]
pub enum RegisterScope {
    Dot,
    Macro(char),
}

impl Hash for RegisterScope {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            RegisterScope::Dot => state.write_u8(2),
            RegisterScope::Macro(_) => state.write_u8(4),
        }
    }
}

impl PartialEq for RegisterScope {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (RegisterScope::Dot, RegisterScope::Dot)
                | (RegisterScope::Macro(_), RegisterScope::Macro(_))
        )
    }
}
