use std::path::PathBuf;

use ratatui::style::Color;
use yeet_buffer::model::viewport::WindowSettings;

/// Colors used by the rendered UI surfaces.
///
/// Defaults mirror the previous hard-coded values so existing output remains unchanged.
#[derive(Clone, Copy, Debug)]
pub struct ThemePalette {
    /// Background color for the active tab label.
    pub tab_active_bg: Color,
    /// Foreground color for the active tab label.
    pub tab_active_fg: Color,
    /// Background color for inactive tab labels.
    pub tab_inactive_bg: Color,
    /// Foreground color for inactive tab labels.
    pub tab_inactive_fg: Color,
    /// Background color used to fill the remaining tab bar width.
    pub tab_fill_bg: Color,
    /// Background color for the statusline.
    pub statusline_bg: Color,
    /// Foreground color for primary statusline labels.
    pub statusline_fg: Color,
    /// Foreground color for secondary or dim statusline text.
    pub statusline_dim_fg: Color,
    /// Foreground color for statusline borders.
    pub statusline_border_fg: Color,
    /// Foreground color for success markers in the statusline.
    pub statusline_success_fg: Color,
    /// Foreground color for warning markers in the statusline.
    pub statusline_warning_fg: Color,
    /// Foreground color for error markers in the statusline.
    pub statusline_error_fg: Color,
}

impl Default for ThemePalette {
    fn default() -> Self {
        Self {
            tab_active_bg: Color::LightBlue,
            tab_active_fg: Color::Black,
            tab_inactive_bg: Color::DarkGray,
            tab_inactive_fg: Color::White,
            tab_fill_bg: Color::Black,
            statusline_bg: Color::Black,
            statusline_fg: Color::White,
            statusline_dim_fg: Color::Gray,
            statusline_border_fg: Color::Black,
            statusline_success_fg: Color::Green,
            statusline_warning_fg: Color::Yellow,
            statusline_error_fg: Color::Red,
        }
    }
}

#[derive(Debug)]
pub struct Settings {
    pub current: WindowSettings,
    pub parent: WindowSettings,
    pub preview: WindowSettings,
    pub theme: ThemePalette,
    pub selection_to_file_on_open: Option<PathBuf>,
    pub selection_to_stdout_on_open: bool,
    pub show_quickfix_signs: bool,
    pub show_mark_signs: bool,
    pub startup_path: Option<PathBuf>,
    pub lua_config_path: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            current: WindowSettings {
                sign_column_width: 2,
            },
            parent: WindowSettings::default(),
            preview: WindowSettings::default(),
            theme: ThemePalette::default(),
            selection_to_file_on_open: None,
            selection_to_stdout_on_open: false,
            show_mark_signs: true,
            show_quickfix_signs: true,
            startup_path: None,
            lua_config_path: None,
        }
    }
}
