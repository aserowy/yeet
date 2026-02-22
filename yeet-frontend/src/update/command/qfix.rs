use std::path::Path;

use yeet_keymap::message::{KeymapMessage, PrintContent};

use crate::{
    action::{self, Action},
    model::{
        qfix::{CdoState, QuickFix, QFIX_SIGN_ID},
        App, Buffer,
    },
    update::{app, sign},
};

pub fn reset(qfix: &mut QuickFix, buffers: Vec<&mut Buffer>) -> Vec<Action> {
    qfix.entries.clear();
    qfix.current_index = 0;
    sign::unset_sign_on_all_buffers(buffers, QFIX_SIGN_ID);

    Vec::new()
}

pub fn clear_in(app: &mut App, qfix: &mut QuickFix, path: &str) -> Vec<Action> {
    let (_, buffer) = app::get_focused_current_mut(app);
    let buffer = match buffer {
        Buffer::Directory(it) => it,
        Buffer::Image(_) => return Vec::new(),
        Buffer::Content(_) => return Vec::new(),
        Buffer::PathReference(_) => return Vec::new(),
        Buffer::Empty => return Vec::new(),
    };

    let path = Path::new(path);
    let current_path = buffer.path.clone().join(path);

    tracing::debug!("clearing current cl for path: {:?}", current_path);

    let mut removed_paths = Vec::new();
    for bl in buffer.buffer.lines.iter_mut() {
        if bl.content.is_empty() {
            continue;
        }

        let path = current_path.join(bl.content.to_stripped_string());
        if qfix.entries.contains(&path) {
            qfix.entries.retain(|p| p != &path);
            removed_paths.push(path);
        }
    }

    sign::unset_sign_for_paths(
        app.buffers.values_mut().collect(),
        removed_paths,
        QFIX_SIGN_ID,
    );

    Vec::new()
}

pub fn cdo(qfix: &mut QuickFix, command: &str) -> Vec<Action> {
    tracing::debug!("cdo command set: {:?}", command);

    qfix.cdo = CdoState::Cdo(None, command.to_owned());

    vec![action::emit_keymap(KeymapMessage::ExecuteCommandString(
        "cfirst".to_string(),
    ))]
}

pub fn select_first(qfix: &mut QuickFix) -> Vec<Action> {
    qfix.current_index = 0;

    match qfix.entries.first() {
        Some(it) => {
            if it.exists() {
                vec![action::emit_keymap(KeymapMessage::NavigateToPathAsPreview(
                    it.clone(),
                ))]
            } else {
                next(qfix)
            }
        }
        None => vec![action::emit_keymap(KeymapMessage::Print(vec![
            PrintContent::Error("no more items".to_owned()),
        ]))],
    }
}

pub fn next(qfix: &mut QuickFix) -> Vec<Action> {
    tracing::debug!(
        "qfix::next called, current_index: {}, entries_count: {}, entries: {:?}",
        qfix.current_index,
        qfix.entries.len(),
        qfix.entries
    );

    let mut entry = qfix.entries.iter().enumerate().filter_map(|(i, p)| {
        let exists = p.exists();
        if i > qfix.current_index && exists {
            tracing::trace!(
                "qfix::next candidate: index={}, path={:?}, exists={}",
                i,
                p,
                exists
            );
            Some((i, p))
        } else {
            if i > qfix.current_index {
                tracing::trace!(
                    "qfix::next skipping: index={}, path={:?}, exists={}",
                    i,
                    p,
                    exists
                );
            }
            None
        }
    });

    match entry.next() {
        Some((i, p)) => {
            tracing::debug!("qfix::next found entry: index={}, path={:?}", i, p);
            qfix.current_index = i;
            vec![action::emit_keymap(KeymapMessage::NavigateToPathAsPreview(
                p.clone(),
            ))]
        }
        None => {
            tracing::debug!("qfix::next no more items found");
            vec![action::emit_keymap(KeymapMessage::Print(vec![
                PrintContent::Error("no more items".to_owned()),
            ]))]
        }
    }
}

pub fn previous(qfix: &mut QuickFix) -> Vec<Action> {
    let mut entry = qfix.entries.iter().enumerate().rev().filter_map(|(i, p)| {
        if i < qfix.current_index && p.exists() {
            Some((i, p))
        } else {
            None
        }
    });

    match entry.next() {
        Some((i, p)) => {
            qfix.current_index = i;
            vec![action::emit_keymap(KeymapMessage::NavigateToPathAsPreview(
                p.clone(),
            ))]
        }
        None => {
            vec![action::emit_keymap(KeymapMessage::Print(vec![
                PrintContent::Error("no more items".to_owned()),
            ]))]
        }
    }
}

pub fn invert_in_current(app: &mut App, qfix: &mut QuickFix) -> Vec<Action> {
    let (_, buffer) = app::get_focused_current_mut(app);
    let buffer = match buffer {
        Buffer::Directory(it) => it,
        Buffer::Image(_) => return Vec::new(),
        Buffer::Content(_) => return Vec::new(),
        Buffer::PathReference(_) => return Vec::new(),
        Buffer::Empty => return Vec::new(),
    };

    let mut added_paths = Vec::new();
    let mut removed_paths = Vec::new();

    let current_path = buffer.path.clone();
    for bl in buffer.buffer.lines.iter_mut() {
        if bl.content.is_empty() {
            continue;
        }

        let path = current_path.join(bl.content.to_stripped_string());
        if qfix.entries.contains(&path) {
            qfix.entries.retain(|p| p != &path);
            removed_paths.push(path);
        } else {
            qfix.entries.push(path.clone());
            added_paths.push(path);
        }
    }

    sign::set_sign_for_paths(
        app.buffers.values_mut().collect(),
        added_paths,
        QFIX_SIGN_ID,
    );

    sign::unset_sign_for_paths(
        app.buffers.values_mut().collect(),
        removed_paths,
        QFIX_SIGN_ID,
    );

    Vec::new()
}
