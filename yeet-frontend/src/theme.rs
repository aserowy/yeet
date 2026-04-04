use std::collections::HashMap;

use ratatui::style::{Color, Style};

pub mod tokens {
    // Tabbar
    pub const TABBAR_ACTIVE_BG: &str = "TabBarActiveBg";
    pub const TABBAR_ACTIVE_FG: &str = "TabBarActiveFg";
    pub const TABBAR_INACTIVE_BG: &str = "TabBarInactiveBg";
    pub const TABBAR_INACTIVE_FG: &str = "TabBarInactiveFg";
    pub const TABBAR_BG: &str = "TabBarBg";

    // Statusline
    pub const STATUSLINE_FOCUSED_FG: &str = "StatusLineFocusedFg";
    pub const STATUSLINE_UNFOCUSED_FG: &str = "StatusLineUnfocusedFg";
    pub const STATUSLINE_BG: &str = "StatusLineBg";
    pub const STATUSLINE_POSITION_FG: &str = "StatusLinePositionFg";
    pub const STATUSLINE_BORDER_BG: &str = "StatusLineBorderBg";
    pub const STATUSLINE_BORDER_FG: &str = "StatusLineBorderFg";
    pub const STATUSLINE_PERMISSIONS_FG: &str = "StatusLinePermissionsFg";

    // Diff indicators
    pub const DIFF_ADDED: &str = "DiffAdded";
    pub const DIFF_MODIFIED: &str = "DiffModified";
    pub const DIFF_REMOVED: &str = "DiffRemoved";

    // Buffer
    pub const CURSOR_LINE_BG: &str = "CursorLineBg";
    pub const SEARCH_BG: &str = "SearchBg";
    pub const LINE_NR: &str = "LineNr";
    pub const CUR_LINE_NR: &str = "CurLineNr";
    pub const BUFFER_BG: &str = "BufferBg";
    pub const BUFFER_FILE_FG: &str = "BufferFileFg";
    pub const BUFFER_DIRECTORY_FG: &str = "BufferDirectoryFg";

    // Directory window borders
    pub const DIRECTORY_BORDER_FG: &str = "DirectoryBorderFg";
    pub const DIRECTORY_BORDER_BG: &str = "DirectoryBorderBg";

    // Split borders
    pub const SPLIT_BORDER_FG: &str = "SplitBorderFg";
    pub const SPLIT_BORDER_BG: &str = "SplitBorderBg";

    // Cursor
    pub const CURSOR_NORMAL: &str = "CursorNormal";
    pub const CURSOR_INSERT: &str = "CursorInsert";

    // Signs
    pub const SIGN_QFIX: &str = "SignQfix";
    pub const SIGN_MARK: &str = "SignMark";

    // Syntax
    pub const SYNTAX_THEME: &str = "syntax";
}

#[derive(Debug, Clone)]
pub struct Theme {
    colors: HashMap<String, Color>,
    pub syntax_theme: String,
}

impl Default for Theme {
    fn default() -> Self {
        let mut colors = HashMap::new();

        // Tabbar defaults (match current hardcoded values)
        colors.insert(tokens::TABBAR_ACTIVE_BG.to_string(), Color::LightBlue);
        colors.insert(tokens::TABBAR_ACTIVE_FG.to_string(), Color::Black);
        colors.insert(tokens::TABBAR_INACTIVE_BG.to_string(), Color::DarkGray);
        colors.insert(tokens::TABBAR_INACTIVE_FG.to_string(), Color::White);
        colors.insert(tokens::TABBAR_BG.to_string(), Color::Black);

        // Statusline defaults
        colors.insert(tokens::STATUSLINE_FOCUSED_FG.to_string(), Color::White);
        colors.insert(tokens::STATUSLINE_UNFOCUSED_FG.to_string(), Color::Gray);
        colors.insert(tokens::STATUSLINE_BG.to_string(), Color::Black);
        colors.insert(tokens::STATUSLINE_POSITION_FG.to_string(), Color::Gray);
        colors.insert(tokens::STATUSLINE_BORDER_FG.to_string(), Color::Black);

        // Diff defaults
        colors.insert(tokens::DIFF_ADDED.to_string(), Color::Green);
        colors.insert(tokens::DIFF_MODIFIED.to_string(), Color::Yellow);
        colors.insert(tokens::DIFF_REMOVED.to_string(), Color::Red);

        // Buffer defaults
        colors.insert(tokens::BUFFER_BG.to_string(), Color::Reset);
        colors.insert(
            tokens::CURSOR_LINE_BG.to_string(),
            Color::Rgb(128, 128, 128),
        );
        colors.insert(tokens::SEARCH_BG.to_string(), Color::Red);
        colors.insert(tokens::LINE_NR.to_string(), Color::Rgb(128, 128, 128));
        colors.insert(tokens::CUR_LINE_NR.to_string(), Color::White);
        colors.insert(tokens::BUFFER_FILE_FG.to_string(), Color::White);
        colors.insert(tokens::BUFFER_DIRECTORY_FG.to_string(), Color::LightBlue);
        colors.insert(tokens::STATUSLINE_PERMISSIONS_FG.to_string(), Color::Gray);
        colors.insert(tokens::STATUSLINE_BORDER_BG.to_string(), Color::Black);
        colors.insert(tokens::DIRECTORY_BORDER_FG.to_string(), Color::Black);
        colors.insert(tokens::DIRECTORY_BORDER_BG.to_string(), Color::Reset);
        colors.insert(tokens::SPLIT_BORDER_FG.to_string(), Color::Black);
        colors.insert(tokens::SPLIT_BORDER_BG.to_string(), Color::Reset);

        // Signs defaults
        colors.insert(tokens::SIGN_QFIX.to_string(), Color::Rgb(255, 85, 255));
        colors.insert(tokens::SIGN_MARK.to_string(), Color::Rgb(85, 255, 255));

        Self {
            colors,
            syntax_theme: "base16-eighties.dark".to_string(),
        }
    }
}

impl Theme {
    pub fn color(&self, token: &str) -> Color {
        self.colors.get(token).copied().unwrap_or(Color::Reset)
    }

    pub fn style_fg(&self, token: &str) -> Style {
        Style::default().fg(self.color(token))
    }

    pub fn style_bg(&self, token: &str) -> Style {
        Style::default().bg(self.color(token))
    }

    pub fn style_fg_bg(&self, fg_token: &str, bg_token: &str) -> Style {
        Style::default()
            .fg(self.color(fg_token))
            .bg(self.color(bg_token))
    }

    pub fn ansi_fg(&self, token: &str) -> String {
        color_to_ansi_fg(self.color(token))
    }

    pub fn ansi_bg(&self, token: &str) -> String {
        color_to_ansi_bg(self.color(token))
    }

    pub fn set_color(&mut self, token: String, color: Color) {
        self.colors.insert(token, color);
    }
}

pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

fn color_to_ansi_fg(color: Color) -> String {
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

fn color_to_ansi_bg(color: Color) -> String {
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

impl Theme {
    pub fn to_buffer_theme_with_border(
        &self,
        border_fg_token: &str,
        border_bg_token: &str,
    ) -> yeet_buffer::BufferTheme {
        yeet_buffer::BufferTheme {
            buffer_bg: self.color(tokens::BUFFER_BG),
            cursor_line_bg: self.color(tokens::CURSOR_LINE_BG),
            search_bg: self.color(tokens::SEARCH_BG),
            line_nr: self.color(tokens::LINE_NR),
            cur_line_nr: self.color(tokens::CUR_LINE_NR),
            border_fg: self.color(border_fg_token),
            border_bg: self.color(border_bg_token),
        }
    }

    pub fn sign_qfix_style(&self) -> String {
        format!("\x1b[1m{}", self.ansi_fg(tokens::SIGN_QFIX))
    }

    pub fn sign_mark_style(&self) -> String {
        format!("\x1b[1m{}", self.ansi_fg(tokens::SIGN_MARK))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#ff0000"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_hex_color("#00ff00"), Some(Color::Rgb(0, 255, 0)));
        assert_eq!(parse_hex_color("#0000ff"), Some(Color::Rgb(0, 0, 255)));
        assert_eq!(parse_hex_color("#1a2b3c"), Some(Color::Rgb(26, 43, 60)));
    }

    #[test]
    fn parse_hex_color_valid_no_hash() {
        assert_eq!(parse_hex_color("ff0000"), Some(Color::Rgb(255, 0, 0)));
    }

    #[test]
    fn parse_hex_color_case_insensitive() {
        assert_eq!(parse_hex_color("#FF00AA"), Some(Color::Rgb(255, 0, 170)));
        assert_eq!(parse_hex_color("#ff00aa"), Some(Color::Rgb(255, 0, 170)));
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert_eq!(parse_hex_color("not-a-color"), None);
        assert_eq!(parse_hex_color("#fff"), None);
        assert_eq!(parse_hex_color("#gggggg"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn theme_style_fg_returns_correct_style() {
        let theme = Theme::default();
        let style = theme.style_fg(tokens::STATUSLINE_FOCUSED_FG);
        assert_eq!(style, Style::default().fg(Color::White));
    }

    #[test]
    fn theme_style_fg_bg_returns_correct_style() {
        let theme = Theme::default();
        let style = theme.style_fg_bg(tokens::TABBAR_ACTIVE_FG, tokens::TABBAR_ACTIVE_BG);
        assert_eq!(
            style,
            Style::default().fg(Color::Black).bg(Color::LightBlue)
        );
    }

    #[test]
    fn theme_ansi_fg_rgb() {
        let mut theme = Theme::default();
        theme.set_color("test".to_string(), Color::Rgb(10, 20, 30));
        assert_eq!(theme.ansi_fg("test"), "\x1b[38;2;10;20;30m");
    }

    #[test]
    fn theme_ansi_bg_rgb() {
        let mut theme = Theme::default();
        theme.set_color("test".to_string(), Color::Rgb(10, 20, 30));
        assert_eq!(theme.ansi_bg("test"), "\x1b[48;2;10;20;30m");
    }

    #[test]
    fn theme_fallback_for_unknown_token() {
        let theme = Theme::default();
        assert_eq!(theme.color("nonexistent"), Color::Reset);
    }

    #[test]
    fn theme_custom_override() {
        let mut theme = Theme::default();
        theme.set_color(tokens::TABBAR_ACTIVE_BG.to_string(), Color::Rgb(255, 0, 0));
        assert_eq!(theme.color(tokens::TABBAR_ACTIVE_BG), Color::Rgb(255, 0, 0));
    }

    #[test]
    fn buffer_bg_default_is_reset() {
        let theme = Theme::default();
        assert_eq!(theme.color(tokens::BUFFER_BG), Color::Reset);
    }

    #[test]
    fn buffer_theme_conversion() {
        let theme = Theme::default();
        let bt =
            theme.to_buffer_theme_with_border(tokens::SPLIT_BORDER_FG, tokens::SPLIT_BORDER_BG);
        assert_eq!(bt.buffer_bg, Color::Reset);
        assert_eq!(bt.cursor_line_bg, Color::Rgb(128, 128, 128));
        assert_eq!(bt.search_bg, Color::Red);
        assert_eq!(bt.line_nr, Color::Rgb(128, 128, 128));
        assert_eq!(bt.cur_line_nr, Color::White);
    }

    #[test]
    fn sign_styles() {
        let theme = Theme::default();
        assert!(theme.sign_qfix_style().contains("\x1b[1m"));
        assert!(theme.sign_mark_style().contains("\x1b[1m"));
    }

    #[test]
    fn default_syntax_theme() {
        let theme = Theme::default();
        assert_eq!(theme.syntax_theme, "base16-eighties.dark");
    }

    #[test]
    fn new_token_defaults_match_current_hardcoded_appearance() {
        let theme = Theme::default();

        // BufferFileFg default is White
        assert_eq!(theme.color(tokens::BUFFER_FILE_FG), Color::White);

        // BufferDirectoryFg default is LightBlue (matches hardcoded \x1b[94m])
        assert_eq!(theme.color(tokens::BUFFER_DIRECTORY_FG), Color::LightBlue);
        assert_eq!(theme.ansi_fg(tokens::BUFFER_DIRECTORY_FG), "\x1b[94m");

        // StatusLinePermissionsFg default is Gray
        assert_eq!(theme.color(tokens::STATUSLINE_PERMISSIONS_FG), Color::Gray);

        // StatusLineBorderBg default is Black
        assert_eq!(theme.color(tokens::STATUSLINE_BORDER_BG), Color::Black);

        // DirectoryBorderFg/Bg defaults
        assert_eq!(theme.color(tokens::DIRECTORY_BORDER_FG), Color::Black);
        assert_eq!(theme.color(tokens::DIRECTORY_BORDER_BG), Color::Reset);

        // SplitBorderFg/Bg defaults (SplitBorderFg replaces old BorderFg=Black)
        assert_eq!(theme.color(tokens::SPLIT_BORDER_FG), Color::Black);
        assert_eq!(theme.color(tokens::SPLIT_BORDER_BG), Color::Reset);
    }

    #[test]
    fn buffer_entry_foreground_color_application() {
        let theme = Theme::default();

        // File foreground ANSI code should be White (\x1b[37m])
        assert_eq!(theme.ansi_fg(tokens::BUFFER_FILE_FG), "\x1b[37m");

        // Directory foreground ANSI code should be LightBlue (\x1b[94m])
        assert_eq!(theme.ansi_fg(tokens::BUFFER_DIRECTORY_FG), "\x1b[94m");

        // Custom override should produce correct ANSI
        let mut custom = Theme::default();
        custom.set_color(
            tokens::BUFFER_DIRECTORY_FG.to_string(),
            Color::Rgb(0, 255, 0),
        );
        assert_eq!(
            custom.ansi_fg(tokens::BUFFER_DIRECTORY_FG),
            "\x1b[38;2;0;255;0m"
        );
    }

    #[test]
    fn statusline_permissions_and_border_background_styling() {
        let theme = Theme::default();

        // Permissions fg style
        let perm_style = theme.style_fg(tokens::STATUSLINE_PERMISSIONS_FG);
        assert_eq!(perm_style, Style::default().fg(Color::Gray));

        // Statusline border combined fg+bg style
        let border_style =
            theme.style_fg_bg(tokens::STATUSLINE_BORDER_FG, tokens::STATUSLINE_BORDER_BG);
        assert_eq!(
            border_style,
            Style::default().fg(Color::Black).bg(Color::Black)
        );
    }

    #[test]
    fn buffer_theme_with_directory_border_tokens() {
        let theme = Theme::default();
        let bt = theme
            .to_buffer_theme_with_border(tokens::DIRECTORY_BORDER_FG, tokens::DIRECTORY_BORDER_BG);
        assert_eq!(bt.border_fg, Color::Black);
        assert_eq!(bt.border_bg, Color::Reset);
    }

    #[test]
    fn buffer_theme_with_split_border_tokens() {
        let theme = Theme::default();
        let bt =
            theme.to_buffer_theme_with_border(tokens::SPLIT_BORDER_FG, tokens::SPLIT_BORDER_BG);
        assert_eq!(bt.border_fg, Color::Black);
        assert_eq!(bt.border_bg, Color::Reset);
    }

    #[test]
    fn old_border_fg_token_does_not_exist() {
        let theme = Theme::default();
        // The old "BorderFg" token is removed — looking it up returns the fallback
        assert_eq!(theme.color("BorderFg"), Color::Reset);
    }
}
