use crate::model::buffer::Buffer;

pub fn get_lines(model: &Buffer) -> Vec<String> {
    model
        .lines
        .iter()
        .skip(model.view_port.vertical_index)
        .take(model.view_port.height)
        .map(|line| correct_line_length(line, model.view_port.width))
        .collect()
}

fn correct_line_length(line: &str, width: usize) -> String {
    let mut line = line.to_string();
    if line.chars().count() > width {
        line = line.chars().take(width).collect();
    }

    line
}
