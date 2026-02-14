use yeet_buffer::{
    message::{BufferMessage, TextModification},
    model::Mode,
};

use crate::{
    action::Action,
    model::{App, Buffer, FileTreeBufferSectionBuffer},
};

use super::app;

pub fn buffer(
    app: &mut App,
    mode: &Mode,
    repeat: &usize,
    modification: &TextModification,
) -> Vec<Action> {
    let msg = BufferMessage::Modification(*repeat, modification.clone());
    match app::get_focused_mut(app) {
        (vp, Buffer::FileTree(it)) => {
            yeet_buffer::update(
                Some(vp),
                mode,
                &mut it.current.buffer,
                std::slice::from_ref(&msg),
            );

            // FIX: only if selection changed!
            it.preview = FileTreeBufferSectionBuffer::None;
        }
        (_vp, Buffer::_Text(_)) => todo!(),
    }
    Vec::new()
}
