use std::collections::HashMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct History {
    pub entries: HashMap<String, HistoryNode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryNode {
    pub changed_at: u64,
    pub component: String,
    pub nodes: HashMap<String, HistoryNode>,
    pub state: HistoryState,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum HistoryState {
    Added,
    #[default]
    Loaded,
}
