use std::path::Path;

use yeet_keymap::message::{KeymapMessage, PrintContent};

use crate::{
    action::{self, Action},
    model::{
        qfix::{CdoState, QFIX_SIGN_ID},
        Model,
    },
    update::sign,
};

pub fn reset(model: &mut Model) -> Vec<Action> {
    model.qfix.entries.clear();
    model.qfix.current_index = 0;
    sign::unset_sign_on_all_buffers(model, QFIX_SIGN_ID);

    Vec::new()
}

pub fn clear_in(model: &mut Model, path: &str) -> Vec<Action> {
    let path = Path::new(path);
    let current_path = model.files.current.path.clone().join(path);

    tracing::debug!("clearing current cl for path: {:?}", current_path);

    for bl in model.files.current.buffer.lines.iter_mut() {
        if bl.content.is_empty() {
            continue;
        }

        let path = current_path.join(bl.content.to_stripped_string());
        if model.qfix.entries.contains(&path) {
            model.qfix.entries.retain(|p| p != &path);
            sign::unset(bl, QFIX_SIGN_ID);
        }
    }

    Vec::new()
}

pub fn cdo(model: &mut Model, command: &str) -> Vec<Action> {
    tracing::debug!("cdo command set: {:?}", command);

    model.qfix.cdo = CdoState::Cdo(None, command.to_owned());

    vec![action::emit_keymap(KeymapMessage::ExecuteCommandString(
        "cfirst".to_string(),
    ))]
}

pub fn select_first(model: &mut Model) -> Vec<Action> {
    model.qfix.current_index = 0;

    match model.qfix.entries.first() {
        Some(it) => {
            if it.exists() {
                vec![action::emit_keymap(KeymapMessage::NavigateToPathAsPreview(
                    it.clone(),
                ))]
            } else {
                next(model)
            }
        }
        None => vec![action::emit_keymap(KeymapMessage::Print(vec![
            PrintContent::Error("no more items".to_owned()),
        ]))],
    }
}

pub fn next(model: &mut Model) -> Vec<Action> {
    let mut entry = model.qfix.entries.iter().enumerate().filter_map(|(i, p)| {
        if i > model.qfix.current_index && p.exists() {
            Some((i, p))
        } else {
            None
        }
    });

    match entry.next() {
        Some((i, p)) => {
            model.qfix.current_index = i;
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

pub fn previous(model: &mut Model) -> Vec<Action> {
    let mut entry = model
        .qfix
        .entries
        .iter()
        .enumerate()
        .rev()
        .filter_map(|(i, p)| {
            if i < model.qfix.current_index && p.exists() {
                Some((i, p))
            } else {
                None
            }
        });

    match entry.next() {
        Some((i, p)) => {
            model.qfix.current_index = i;
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

pub fn invert_in_current(model: &mut Model) -> Vec<Action> {
    let current_path = model.files.current.path.clone();
    for bl in model.files.current.buffer.lines.iter_mut() {
        if bl.content.is_empty() {
            continue;
        }

        let path = current_path.join(bl.content.to_stripped_string());
        if model.qfix.entries.contains(&path) {
            model.qfix.entries.retain(|p| p != &path);
            sign::unset(bl, QFIX_SIGN_ID);
        } else {
            model.qfix.entries.push(path.clone());
            sign::set(bl, QFIX_SIGN_ID);
        }
    }

    Vec::new()
}
