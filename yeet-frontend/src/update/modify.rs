use yeet_buffer::{
    message::{BufferMessage, TextModification},
    model::Mode,
};

use crate::{
    action::Action,
    model::{FileTreeBuffer, FileTreeBufferSectionBuffer},
};

pub fn buffer(
    mode: &Mode,
    buffer: &mut FileTreeBuffer,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let msg = BufferMessage::Modification(*repeat, modification.clone());

    yeet_buffer::update::update_buffer(
        &mut buffer.current_vp,
        &mut buffer.current_cursor,
        mode,
        &mut buffer.current.buffer,
        &msg,
    );

    // FIX: only if selection changed!
    buffer.preview = FileTreeBufferSectionBuffer::None;

    Vec::new()
}
