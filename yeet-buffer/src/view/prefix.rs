use std::cmp::Reverse;

use crate::{
    model::{
        ansi::Ansi,
        viewport::{LineNumber, ViewPort},
        BufferLine, Cursor,
    },
    BufferTheme,
};

use super::style::{self, CUR_LINE_NR_BOLD};

pub fn get_border(vp: &ViewPort) -> Ansi {
    Ansi::new(&" ".repeat(vp.get_border_width()))
}

pub fn get_custom_prefix(line: &BufferLine) -> Ansi {
    if let Some(prefix) = &line.prefix {
        Ansi::new(prefix)
    } else {
        Ansi::new("")
    }
}

pub fn get_line_number(vp: &ViewPort, index: usize, cursor: &Cursor, theme: &BufferTheme) -> Ansi {
    if vp.line_number == LineNumber::None {
        return Ansi::new("");
    }

    let width = vp.get_line_number_width();
    let number = {
        let number_string = (index + 1).to_string();
        if let Some(index) = number_string.char_indices().nth_back(width - 1) {
            number_string[index.0..].to_string()
        } else {
            number_string
        }
    };

    if cursor.vertical_index == index {
        let cur_line_nr_fg = style::color_to_ansi_fg(theme.cur_line_nr);
        return Ansi::new(&format!(
            "{}{}{:<width$}\x1b[0m",
            CUR_LINE_NR_BOLD, cur_line_nr_fg, number
        ));
    }

    match vp.line_number {
        LineNumber::Absolute => Ansi::new(&format!("{:>width$} ", number)),
        LineNumber::None => Ansi::new(""),
        LineNumber::Relative => {
            let relative = cursor.vertical_index.abs_diff(index);
            let line_nr_fg = style::color_to_ansi_fg(theme.line_nr);

            Ansi::new(&format!("{}{:>width$}\x1b[0m", line_nr_fg, relative))
        }
    }
}

pub fn get_signs(vp: &ViewPort, bl: &BufferLine) -> Ansi {
    let max_sign_count = vp.sign_column_width;

    let mut filtered: Vec<_> = bl
        .signs
        .iter()
        .filter(|s| !vp.hidden_sign_ids.contains(&s.id))
        .collect();

    filtered.sort_unstable_by_key(|s| Reverse(s.priority));

    let signs_string = filtered
        .iter()
        .take(max_sign_count)
        .fold("".to_string(), |acc, s| {
            format!("{}{}{}\x1b[0m", acc, s.style, s.content)
        });

    let signs = Ansi::new(&signs_string);
    let char_count = signs.count_chars();
    if char_count < max_sign_count {
        Ansi::new(&format!(
            "{}{}",
            signs,
            " ".repeat(max_sign_count - char_count)
        ))
    } else {
        signs
    }
}
