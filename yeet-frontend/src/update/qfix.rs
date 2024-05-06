use std::path::Path;

use ratatui::style::Color;
use yeet_buffer::model::{BufferLine, Sign, StylePartial};

use crate::{
    action::Action,
    model::{
        qfix::{QuickFix, QFIX_SIGN_ID},
        Model,
    },
};

use super::{
    selection::{get_current_selected_bufferline, get_current_selected_path},
    sign::{set_sign, unset_sign},
};

pub fn toggle_selected_to_qfix(model: &mut Model) -> Vec<Action> {
    let selected = get_current_selected_path(model);
    if let Some(selected) = selected {
        if model.qfix.entries.contains(&selected) {
            model.qfix.entries.retain(|p| p != &selected);
            if let Some(bl) = get_current_selected_bufferline(model) {
                unset_sign(bl, QFIX_SIGN_ID);
            }
        } else {
            model.qfix.entries.push(selected);
            if let Some(bl) = get_current_selected_bufferline(model) {
                set_sign(bl, generate_qfix_sign());
            }
        }
    }
    Vec::new()
}

pub fn set_sign_if_qfix(qfix: &QuickFix, bl: &mut BufferLine, path: &Path) {
    let is_marked = qfix.entries.iter().any(|p| p == path);
    if !is_marked {
        return;
    }

    set_sign(bl, generate_qfix_sign());
}

fn generate_qfix_sign() -> Sign {
    Sign {
        id: QFIX_SIGN_ID,
        content: 'c',
        priority: 0,
        style: vec![StylePartial::Foreground(Color::LightMagenta)],
    }
}
