use yeet_buffer::model::SignIdentifier;

use crate::model::{mark::MARK_SIGN_ID, qfix::QFIX_SIGN_ID, Model};

pub fn update_with_settings(model: &mut Model) {
    model.buffer.current_vp.set(&model.settings.current);
    model.buffer.parent_vp.set(&model.settings.current);
    model.buffer.preview_vp.set(&model.settings.current);

    if model.settings.show_mark_signs {
        remove_hidden_sign_on_all_buffer(model, &MARK_SIGN_ID);
    } else {
        add_hidden_sign_on_all_buffer(model, MARK_SIGN_ID);
    }

    if model.settings.show_quickfix_signs {
        remove_hidden_sign_on_all_buffer(model, &QFIX_SIGN_ID);
    } else {
        add_hidden_sign_on_all_buffer(model, QFIX_SIGN_ID);
    }
}

fn add_hidden_sign_on_all_buffer(model: &mut Model, id: SignIdentifier) {
    model.buffer.current_vp.hidden_sign_ids.insert(id);
    model.buffer.parent_vp.hidden_sign_ids.insert(id);
    model.buffer.preview_vp.hidden_sign_ids.insert(id);
}

fn remove_hidden_sign_on_all_buffer(model: &mut Model, id: &SignIdentifier) {
    model.buffer.current_vp.hidden_sign_ids.remove(id);
    model.buffer.parent_vp.hidden_sign_ids.remove(id);
    model.buffer.preview_vp.hidden_sign_ids.remove(id);
}
