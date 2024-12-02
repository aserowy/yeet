use yeet_buffer::model::{viewport::ViewPort, SignIdentifier};

use crate::model::{mark::MARK_SIGN_ID, qfix::QFIX_SIGN_ID, Model};

pub fn update_with_settings(model: &mut Model) {
    model.files.current_vp.set(&model.settings.current);
    model.files.parent_vp.set(&model.settings.current);
    model.files.preview_vp.set(&model.settings.current);

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
    add_hidden_sign(&mut model.files.current_vp, id);
    add_hidden_sign(&mut model.files.parent_vp, id);
    add_hidden_sign(&mut model.files.preview_vp, id);
}

fn add_hidden_sign(viewport: &mut ViewPort, id: SignIdentifier) {
    viewport.hidden_sign_ids.insert(id);
}

fn remove_hidden_sign_on_all_buffer(model: &mut Model, id: &SignIdentifier) {
    remove_hidden_sign(&mut model.files.current_vp, id);
    remove_hidden_sign(&mut model.files.parent_vp, id);
    remove_hidden_sign(&mut model.files.preview_vp, id);
}

fn remove_hidden_sign(viewport: &mut ViewPort, id: &SignIdentifier) {
    viewport.hidden_sign_ids.remove(id);
}
