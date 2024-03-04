use ratatui::style::Color;
use yeet_keymap::message::{CommandMode, Mode};

use crate::model::{
    buffer::{Buffer, StylePartial, StylePartialSpan},
    Model,
};

// TODO: nohl, n, N, enter
pub fn update(model: &mut Model) {
    let search = match model.commandline.buffer.lines.last() {
        Some(line) => &line.content,
        None => return,
    };

    let _downwards = matches!(model.mode, Mode::Command(CommandMode::SearchDown));

    if model.parent.path.is_some() {
        mark_search_results(&mut model.parent.buffer, search);
    }

    if model.preview.path.is_dir() {
        mark_search_results(&mut model.preview.buffer, search);
    }

    mark_search_results(&mut model.current.buffer, search);
}

pub fn clear(model: &mut Model) {
    for line in &mut model.parent.buffer.lines {
        line.search = None;
    }
    for line in &mut model.preview.buffer.lines {
        line.search = None;
    }
    for line in &mut model.current.buffer.lines {
        line.search = None;
    }
}

fn mark_search_results(buffer: &mut Buffer, search: &str) {
    let len = search.chars().count();
    for line in &mut buffer.lines {
        line.search = None;

        let start = match line.content.find(search) {
            Some(it) => line.content[..it].chars().count(),
            None => continue,
        };

        line.search = Some(vec![
            StylePartialSpan {
                start,
                end: start + len,
                style: StylePartial::Foreground(Color::Black),
            },
            StylePartialSpan {
                start,
                end: start + len,
                style: StylePartial::Background(Color::Magenta),
            },
        ]);
    }
}
