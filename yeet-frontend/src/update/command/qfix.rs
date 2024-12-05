use std::path::Path;

use yeet_keymap::message::{KeymapMessage, PrintContent};

use crate::{
    action::{self, Action},
    model::{
        qfix::{CdoState, QuickFix, QFIX_SIGN_ID},
        FileTreeBuffer,
    },
    update::sign,
};

pub fn reset(qfix: &mut QuickFix, buffer: &mut FileTreeBuffer) -> Vec<Action> {
    qfix.entries.clear();
    qfix.current_index = 0;
    sign::unset_sign_on_all_buffers(buffer, QFIX_SIGN_ID);

    Vec::new()
}

pub fn clear_in(qfix: &mut QuickFix, buffer: &mut FileTreeBuffer, path: &str) -> Vec<Action> {
    let path = Path::new(path);
    let current_path = buffer.current.path.clone().join(path);

    tracing::debug!("clearing current cl for path: {:?}", current_path);

    for bl in buffer.current.buffer.lines.iter_mut() {
        if bl.content.is_empty() {
            continue;
        }

        let path = current_path.join(bl.content.to_stripped_string());
        if qfix.entries.contains(&path) {
            qfix.entries.retain(|p| p != &path);
            sign::unset(bl, QFIX_SIGN_ID);
        }
    }

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
    let mut entry = qfix.entries.iter().enumerate().filter_map(|(i, p)| {
        if i > qfix.current_index && p.exists() {
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

pub fn invert_in_current( qfix: &mut QuickFix,buffer: &mut FileTreeBuffer) -> Vec<Action> {
    let current_path = buffer.current.path.clone();
    for bl in buffer.current.buffer.lines.iter_mut() {
        if bl.content.is_empty() {
            continue;
        }

        let path = current_path.join(bl.content.to_stripped_string());
        if qfix.entries.contains(&path) {
            qfix.entries.retain(|p| p != &path);
            sign::unset(bl, QFIX_SIGN_ID);
        } else {
            qfix.entries.push(path.clone());
            sign::set(bl, QFIX_SIGN_ID);
        }
    }

    Vec::new()
}
