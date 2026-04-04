use ratatui::style::Color;

pub fn ansi_reset_with_bg(bg: Color) -> String {
    format!("\x1b[0m{}", color_to_ansi_bg(bg))
}
pub const CURSOR_NORMAL_CODE: &str = "\x1b[7m";
pub const CURSOR_NORMAL_RESET: &str = "\x1b[27m";
pub const CURSOR_INSERT_CODE: &str = "\x1b[4m";
pub const CURSOR_INSERT_RESET: &str = "\x1b[24m";
pub const CUR_LINE_NR_BOLD: &str = "\x1b[1m";

pub fn color_to_ansi_fg(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("\x1b[38;2;{};{};{}m", r, g, b),
        Color::Indexed(i) => format!("\x1b[38;5;{}m", i),
        Color::Black => "\x1b[30m".to_string(),
        Color::Red => "\x1b[31m".to_string(),
        Color::Green => "\x1b[32m".to_string(),
        Color::Yellow => "\x1b[33m".to_string(),
        Color::Blue => "\x1b[34m".to_string(),
        Color::Magenta => "\x1b[35m".to_string(),
        Color::Cyan => "\x1b[36m".to_string(),
        Color::White => "\x1b[37m".to_string(),
        Color::Gray => "\x1b[90m".to_string(),
        Color::DarkGray => "\x1b[90m".to_string(),
        Color::LightRed => "\x1b[91m".to_string(),
        Color::LightGreen => "\x1b[92m".to_string(),
        Color::LightYellow => "\x1b[93m".to_string(),
        Color::LightBlue => "\x1b[94m".to_string(),
        Color::LightMagenta => "\x1b[95m".to_string(),
        Color::LightCyan => "\x1b[96m".to_string(),
        _ => "\x1b[39m".to_string(),
    }
}

pub fn color_to_ansi_bg(color: Color) -> String {
    match color {
        Color::Rgb(r, g, b) => format!("\x1b[48;2;{};{};{}m", r, g, b),
        Color::Indexed(i) => format!("\x1b[48;5;{}m", i),
        Color::Black => "\x1b[40m".to_string(),
        Color::Red => "\x1b[41m".to_string(),
        Color::Green => "\x1b[42m".to_string(),
        Color::Yellow => "\x1b[43m".to_string(),
        Color::Blue => "\x1b[44m".to_string(),
        Color::Magenta => "\x1b[45m".to_string(),
        Color::Cyan => "\x1b[46m".to_string(),
        Color::White => "\x1b[47m".to_string(),
        Color::Gray => "\x1b[100m".to_string(),
        Color::DarkGray => "\x1b[100m".to_string(),
        Color::LightRed => "\x1b[101m".to_string(),
        Color::LightGreen => "\x1b[102m".to_string(),
        Color::LightYellow => "\x1b[103m".to_string(),
        Color::LightBlue => "\x1b[104m".to_string(),
        Color::LightMagenta => "\x1b[105m".to_string(),
        Color::LightCyan => "\x1b[106m".to_string(),
        _ => "\x1b[49m".to_string(),
    }
}
