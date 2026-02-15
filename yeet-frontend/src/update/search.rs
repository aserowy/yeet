use yeet_buffer::model::TextBuffer;

use crate::model::Buffer;

pub fn search_in_buffers(buffers: Vec<&mut Buffer>, search: Option<String>) {
    let search = match search {
        Some(it) => it,
        None => {
            clear(buffers);
            return;
        }
    };

    for buffer in buffers {
        let buffer = match buffer {
            Buffer::Directory(it) => it,
            Buffer::Image(_) => continue,
            Buffer::Content(_) => continue,
            Buffer::Empty => continue,
        };
        set_search_char_positions(&mut buffer.buffer, search.as_str());
    }
}

pub fn clear(buffers: Vec<&mut Buffer>) {
    for buffer in buffers {
        let buffer = match buffer {
            Buffer::Directory(it) => it,
            Buffer::Image(_) => continue,
            Buffer::Content(_) => continue,
            Buffer::Empty => continue,
        };

        for line in &mut buffer.buffer.lines {
            line.search_char_position = None;
        }
    }
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
