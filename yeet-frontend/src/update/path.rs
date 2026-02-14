use std::path::{Path, PathBuf};

use yeet_buffer::model::Mode;

use crate::{
    action::Action,
    model::{history::History, junkyard::JunkYard, mark::Marks, qfix::QuickFix, Buffer},
};

use super::junkyard::remove_from_junkyard;

#[tracing::instrument(skip(buffers))]
pub fn add(
    history: &History,
    marks: &Marks,
    qfix: &QuickFix,
    mode: &Mode,
    buffers: Vec<&mut Buffer>,
    paths: &[PathBuf],
) -> Vec<Action> {
    let _ = (history, marks, qfix, mode, buffers, paths);
    Vec::new()
}

#[tracing::instrument(skip(junk, buffers))]
pub fn remove(
    history: &History,
    junk: &mut JunkYard,
    mode: &Mode,
    buffers: Vec<&mut Buffer>,
    path: &Path,
) -> Vec<Action> {
    let _ = (buffers, history, mode);
    if path.starts_with(junk.path.clone()) {
        remove_from_junkyard(junk, path);
    }

    Vec::new()
}
