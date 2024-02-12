use yate_keymap::message::Mode;

use crate::model::buffer::Buffer;

pub fn sort_content(mode: &Mode, model: &mut Buffer) {
    model.lines.sort_unstable_by(|a, b| {
        a.content
            .to_ascii_uppercase()
            .cmp(&b.content.to_ascii_uppercase())
    });
    super::buffer::cursor::validate(mode, model);
}
