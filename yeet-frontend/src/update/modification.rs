use yeet_buffer::message::{BufferMessage, TextModification};

use crate::{
    action::Action,
    model::{FileTreeBufferSectionType, Model},
};

pub fn modify_buffer(
    model: &mut Model,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let msg = BufferMessage::Modification(*repeat, modification.clone());
    super::update_current(model, &msg);

    model.files.preview = FileTreeBufferSectionType::None;

    Vec::new()
}
