use yeet_buffer::{
    message::{BufferMessage, TextModification},
    model::Mode,
};

use crate::{
    action::Action,
    layout::AppLayout,
    model::{FileTreeBuffer, FileTreeBufferSectionBuffer, },
};

pub fn modify_buffer(
    layout: &AppLayout,
    mode: &Mode,
    buffer: &mut FileTreeBuffer,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let msg = BufferMessage::Modification(*repeat, modification.clone());
    super::update_current(layout, mode, buffer, &msg);

    // FIX: only if selection changed!
    buffer.preview = FileTreeBufferSectionBuffer::None;

    Vec::new()
}
