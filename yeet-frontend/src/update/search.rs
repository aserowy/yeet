use yeet_buffer::model::TextBuffer;

use crate::{
    action::Action,
    model::{FileTreeBuffer, FileTreeBufferSectionBuffer},
};

pub fn search_in_buffers(buffer: &mut FileTreeBuffer, search: Option<String>) {
    let search = match search {
        Some(it) => it,
        None => {
            clear(buffer);
            return;
        }
    };

    set_search_char_positions(&mut buffer.current.buffer, search.as_str());

    if let FileTreeBufferSectionBuffer::Text(path, text_buffer) = &mut buffer.parent {
        if path.is_dir() {
            set_search_char_positions(text_buffer, search.as_str());
        }
    };

    if let FileTreeBufferSectionBuffer::Text(path, text_buffer) = &mut buffer.preview {
        if path.is_dir() {
            set_search_char_positions(text_buffer, search.as_str());
        }
    };
}

pub fn clear(buffer: &mut FileTreeBuffer) -> Vec<Action> {
    for line in &mut buffer.current.buffer.lines {
        line.search_char_position = None;
    }
    if let FileTreeBufferSectionBuffer::Text(_, text_buffer) = &mut buffer.parent {
        for line in &mut text_buffer.lines {
            line.search_char_position = None;
        }
    }
    if let FileTreeBufferSectionBuffer::Text(_, text_buffer) = &mut buffer.preview {
        for line in &mut text_buffer.lines {
            line.search_char_position = None;
        }
    }
    Vec::new()
}

fn set_search_char_positions(buffer: &mut TextBuffer, search: &str) {
    let smart_case = search.chars().all(|c| c.is_ascii_lowercase());
    let search_length = search.chars().count();

    for line in &mut buffer.lines {
        line.search_char_position = None;

        let mut content = line.content.to_stripped_string();
        let lower = content.to_lowercase();
        if smart_case {
            content = lower;
        };

        let start = match content.find(search) {
            Some(it) => content[..it].chars().count(),
            None => continue,
        };

        line.search_char_position = Some(vec![(start, search_length)]);
    }
}
