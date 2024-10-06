use yeet_keymap::message::{KeymapMessage, PrintContent};

use crate::{
    action::{self, Action},
    model::{
        qfix::{CdoState, QFIX_SIGN_ID},
        Model,
    },
    update::sign::{set_sign, unset_sign, unset_sign_on_all_buffers},
};

pub fn reset_qfix_list(model: &mut Model, additional_action: Action) -> Vec<Action> {
    model.qfix.entries.clear();
    model.qfix.current_index = 0;
    unset_sign_on_all_buffers(model, QFIX_SIGN_ID);

    vec![additional_action]
}

pub fn clear_qfix_list_in_current(model: &mut Model, additional_action: Action) -> Vec<Action> {
    let current_path = model.files.current.path.clone();
    for bl in model.files.current.buffer.lines.iter_mut() {
        if bl.content.is_empty() {
            continue;
        }

        let path = current_path.join(bl.content.to_stripped_string());
        if model.qfix.entries.contains(&path) {
            model.qfix.entries.retain(|p| p != &path);
            unset_sign(bl, QFIX_SIGN_ID);
        }
    }

    vec![additional_action]
}

pub fn cdo(model: &mut Model, command: &str, additional_action: Action) -> Vec<Action> {
    tracing::debug!("cdo command set: {:?}", command);

    model.qfix.cdo = CdoState::Cdo(None, command.to_owned());

    vec![
        additional_action,
        action::emit_keymap(KeymapMessage::ExecuteCommandString("cfirst".to_string())),
    ]
}

pub fn navigate_first_qfix_entry(model: &mut Model, additional_action: Action) -> Vec<Action> {
    model.qfix.current_index = 0;

    match model.qfix.entries.first() {
        Some(it) => {
            if it.exists() {
                vec![
                    additional_action,
                    action::emit_keymap(KeymapMessage::NavigateToPathAsPreview(it.clone())),
                ]
            } else {
                navigate_next_qfix_entry(model, additional_action)
            }
        }
        None => vec![additional_action],
    }
}

pub fn navigate_next_qfix_entry(model: &mut Model, additional_action: Action) -> Vec<Action> {
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
            vec![
                additional_action,
                action::emit_keymap(KeymapMessage::NavigateToPathAsPreview(p.clone())),
            ]
        }
        None => {
            vec![action::emit_keymap(KeymapMessage::Print(vec![
                PrintContent::Error("no more items".to_owned()),
            ]))]
        }
    }
}

pub fn navigate_previous_qfix_entry(model: &mut Model, additional_action: Action) -> Vec<Action> {
    let mut entry = model
        .qfix
        .entries
        .iter()
        .rev()
        .enumerate()
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
            vec![
                additional_action,
                action::emit_keymap(KeymapMessage::NavigateToPathAsPreview(p.clone())),
            ]
        }
        None => {
            vec![action::emit_keymap(KeymapMessage::Print(vec![
                PrintContent::Error("no more items".to_owned()),
            ]))]
        }
    }
}

pub fn invert_qfix_selection_in_current(
    model: &mut Model,
    additional_action: Action,
) -> Vec<Action> {
    let current_path = model.files.current.path.clone();
    for bl in model.files.current.buffer.lines.iter_mut() {
        if bl.content.is_empty() {
            continue;
        }

        let path = current_path.join(bl.content.to_stripped_string());
        if model.qfix.entries.contains(&path) {
            model.qfix.entries.retain(|p| p != &path);
            unset_sign(bl, QFIX_SIGN_ID);
        } else {
            model.qfix.entries.push(path.clone());
            set_sign(bl, QFIX_SIGN_ID);
        }
    }

    vec![additional_action]
}
