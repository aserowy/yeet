use std::{collections::HashMap, path::PathBuf};

use yeet_buffer::model::SignIdentifier;

pub const MARK_SIGN_ID: SignIdentifier = "mark";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Marks {
    pub entries: HashMap<char, PathBuf>,
}
