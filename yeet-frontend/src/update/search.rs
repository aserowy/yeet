use ratatui::style::Color;
use yeet_buffer::model::{Buffer, StylePartial, StylePartialSpan};

use crate::model::Model;

pub fn search(model: &mut Model, search: Option<String>) {
    let search = match search {
        Some(it) => it,
        None => {
            clear(model);
            return;
        }
    };

    if model.files.parent.path.is_some() {
        set_styles(&mut model.files.parent.buffer, search.as_str());
    }

    let is_preview_dir = model
        .files
        .preview
        .path
        .as_ref()
        .is_some_and(|p| p.is_dir());

    if is_preview_dir {
        set_styles(&mut model.files.preview.buffer, search.as_str());
    }

    set_styles(&mut model.files.current.buffer, search.as_str());
}

pub fn clear(model: &mut Model) {
    for line in &mut model.files.parent.buffer.lines {
        line.search = None;
    }
    for line in &mut model.files.preview.buffer.lines {
        line.search = None;
    }
    for line in &mut model.files.current.buffer.lines {
        line.search = None;
    }
}

fn set_styles(buffer: &mut Buffer, search: &str) {
    let len = search.chars().count();
    let smart_case = search.chars().all(|c| c.is_ascii_lowercase());

    for line in &mut buffer.lines {
        line.search = None;

        let mut content = line.content.as_str();

        let lower = content.to_lowercase();
        if smart_case {
            content = lower.as_str();
        };

        let start = match content.find(search) {
            Some(it) => content[..it].chars().count(),
            None => continue,
        };

        line.search = Some(vec![
            StylePartialSpan {
                start,
                end: start + len,
                style: StylePartial::Foreground(Color::DarkGray),
            },
            StylePartialSpan {
                start,
                end: start + len,
                style: StylePartial::Background(Color::Magenta),
            },
        ]);
    }
}
