use std::path::PathBuf;

use yeet_buffer::model::SignIdentifier;

pub const QFIX_SIGN_ID: SignIdentifier = "qfix";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuickFix {
    pub current_index: usize,
    pub cdo: CdoState,
    pub entries: Vec<PathBuf>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CdoState {
    Cdo(Option<usize>, String),
    Cnext(String),
    #[default]
    None,
}
