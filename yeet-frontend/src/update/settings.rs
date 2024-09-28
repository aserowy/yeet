use yeet_buffer::model::{Buffer, SignIdentifier};

use crate::model::{mark::MARK_SIGN_ID, qfix::QFIX_SIGN_ID, Model, PreviewContent};

pub fn update_with_settings(model: &mut Model) {
    model.files.current.buffer.set(&model.settings.current);
    model.files.parent.buffer.set(&model.settings.parent);

    if let PreviewContent::Buffer(dir) = &mut model.files.preview {
        dir.buffer.set(&model.settings.preview);
    }

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
    add_hidden_sign(&mut model.files.current.buffer, id);
    add_hidden_sign(&mut model.files.parent.buffer, id);

    if let PreviewContent::Buffer(dir) = &mut model.files.preview {
        add_hidden_sign(&mut dir.buffer, id);
    }
}

fn add_hidden_sign(buffer: &mut Buffer, id: SignIdentifier) {
    buffer.view_port.hidden_sign_ids.insert(id);
}

fn remove_hidden_sign_on_all_buffer(model: &mut Model, id: &SignIdentifier) {
    remove_hidden_sign(&mut model.files.current.buffer, id);
    remove_hidden_sign(&mut model.files.parent.buffer, id);

    if let PreviewContent::Buffer(dir) = &mut model.files.preview {
        remove_hidden_sign(&mut dir.buffer, id);
    }
}

fn remove_hidden_sign(buffer: &mut Buffer, id: &SignIdentifier) {
    buffer.view_port.hidden_sign_ids.remove(id);
}
