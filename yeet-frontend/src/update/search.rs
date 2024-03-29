use ratatui::style::Color;
use yeet_buffer::model::{Buffer, CommandMode, Mode, SearchModel, StylePartial, StylePartialSpan};

use crate::model::Model;

pub fn update(model: &mut Model) {
    let search = match model.commandline.buffer.lines.last() {
        Some(line) => &line.content,
        None => return,
    };

    let direction = if let Mode::Command(CommandMode::Search(direction)) = &model.mode {
        direction.clone()
    } else {
        return;
    };

    model.search = Some(SearchModel {
        last: search.to_owned(),
        direction,
    });

    super::search::search(model);
}

pub fn search(model: &mut Model) {
    let search = match &model.search {
        Some(it) => it.last.as_str(),
        None => return,
    };

    if model.file_buffer.parent.path.is_some() {
        set_styles(&mut model.file_buffer.parent.buffer, search);
    }

    if model
        .file_buffer
        .preview
        .path
        .as_ref()
        .is_some_and(|p| p.is_dir())
    {
        set_styles(&mut model.file_buffer.preview.buffer, search);
    }

    set_styles(&mut model.file_buffer.current.buffer, search);
}

pub fn clear(model: &mut Model) {
    for line in &mut model.file_buffer.parent.buffer.lines {
        line.search = None;
    }
    for line in &mut model.file_buffer.preview.buffer.lines {
        line.search = None;
    }
    for line in &mut model.file_buffer.current.buffer.lines {
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
