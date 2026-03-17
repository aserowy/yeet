use std::{fs, path::Path};

use mlua::Value;
use ratatui::style::Color;
use tracing::error;

use crate::settings::{Settings, ThemePalette};

#[derive(Clone, Copy, Debug, Default)]
pub struct ThemePaletteOverrides {
    pub tab_active_bg: Option<Color>,
    pub tab_active_fg: Option<Color>,
    pub tab_inactive_bg: Option<Color>,
    pub tab_inactive_fg: Option<Color>,
    pub tab_fill_bg: Option<Color>,
    pub statusline_bg: Option<Color>,
    pub statusline_fg: Option<Color>,
    pub statusline_dim_fg: Option<Color>,
    pub statusline_border_fg: Option<Color>,
    pub statusline_success_fg: Option<Color>,
    pub statusline_warning_fg: Option<Color>,
    pub statusline_error_fg: Option<Color>,
}

pub fn apply_theme_overrides(base: ThemePalette, overrides: ThemePaletteOverrides) -> ThemePalette {
    ThemePalette {
        tab_active_bg: overrides.tab_active_bg.unwrap_or(base.tab_active_bg),
        tab_active_fg: overrides.tab_active_fg.unwrap_or(base.tab_active_fg),
        tab_inactive_bg: overrides.tab_inactive_bg.unwrap_or(base.tab_inactive_bg),
        tab_inactive_fg: overrides.tab_inactive_fg.unwrap_or(base.tab_inactive_fg),
        tab_fill_bg: overrides.tab_fill_bg.unwrap_or(base.tab_fill_bg),
        statusline_bg: overrides.statusline_bg.unwrap_or(base.statusline_bg),
        statusline_fg: overrides.statusline_fg.unwrap_or(base.statusline_fg),
        statusline_dim_fg: overrides
            .statusline_dim_fg
            .unwrap_or(base.statusline_dim_fg),
        statusline_border_fg: overrides
            .statusline_border_fg
            .unwrap_or(base.statusline_border_fg),
        statusline_success_fg: overrides
            .statusline_success_fg
            .unwrap_or(base.statusline_success_fg),
        statusline_warning_fg: overrides
            .statusline_warning_fg
            .unwrap_or(base.statusline_warning_fg),
        statusline_error_fg: overrides
            .statusline_error_fg
            .unwrap_or(base.statusline_error_fg),
    }
}

pub fn load_lua_theme_overrides_from_source(source: &str) -> Result<ThemePaletteOverrides, String> {
    let lua = mlua::Lua::new();
    lua.load(source)
        .set_name("yeet-init.lua")
        .exec()
        .map_err(|err| format!("failed to execute lua config: {err}"))?;
    Ok(read_theme_palette_overrides(&lua))
}

pub fn load_lua_theme_overrides_from_path(path: &Path) -> Result<ThemePaletteOverrides, String> {
    let source = fs::read_to_string(path)
        .map_err(|err| format!("failed to read lua config {}: {err}", path.display()))?;
    load_lua_theme_overrides_from_source(&source)
}

/// Apply Lua theme settings from the config path, if configured.
///
/// Errors are logged and defaults preserved.
pub fn apply_lua_theme_settings(mut settings: Settings) -> Settings {
    let lua_config_path = match settings.lua_config_path.clone() {
        Some(path) => path,
        None => return settings,
    };

    let overrides = match load_lua_theme_overrides_from_path(&lua_config_path) {
        Ok(overrides) => overrides,
        Err(err) => {
            error!(
                "failed to load lua config {}: {err}",
                lua_config_path.display()
            );
            return settings;
        }
    };

    settings.theme = apply_theme_overrides(settings.theme, overrides);
    settings
}

fn read_theme_palette_overrides(lua: &mlua::Lua) -> ThemePaletteOverrides {
    let globals = lua.globals();
    let theme_value = match globals.get::<mlua::Value>("theme") {
        Ok(value) => value,
        Err(err) => {
            error!("failed to read lua theme table: {err}");
            return ThemePaletteOverrides::default();
        }
    };

    let theme_table = match theme_value {
        Value::Nil => return ThemePaletteOverrides::default(),
        Value::Table(table) => table,
        other => {
            error!("lua theme must be a table, got {:?}", other.type_name());
            return ThemePaletteOverrides::default();
        }
    };

    let mut overrides = ThemePaletteOverrides::default();
    let apply_color = |key: &str, target: &mut Option<Color>| match theme_table.get::<Value>(key) {
        Ok(Value::Nil) => {}
        Ok(Value::String(value)) => match parse_lua_color(value.to_str()) {
            Ok(color) => {
                *target = Some(color);
            }
            Err(message) => {
                error!("{message}");
            }
        },
        Ok(other) => {
            error!(
                "lua theme value for '{key}' must be a string, got {:?}",
                other.type_name()
            );
        }
        Err(err) => {
            error!("failed to read lua theme key '{key}': {err}");
        }
    };

    apply_color("tab_active_bg", &mut overrides.tab_active_bg);
    apply_color("tab_active_fg", &mut overrides.tab_active_fg);
    apply_color("tab_inactive_bg", &mut overrides.tab_inactive_bg);
    apply_color("tab_inactive_fg", &mut overrides.tab_inactive_fg);
    apply_color("tab_fill_bg", &mut overrides.tab_fill_bg);
    apply_color("statusline_bg", &mut overrides.statusline_bg);
    apply_color("statusline_fg", &mut overrides.statusline_fg);
    apply_color("statusline_dim_fg", &mut overrides.statusline_dim_fg);
    apply_color("statusline_border_fg", &mut overrides.statusline_border_fg);
    apply_color(
        "statusline_success_fg",
        &mut overrides.statusline_success_fg,
    );
    apply_color(
        "statusline_warning_fg",
        &mut overrides.statusline_warning_fg,
    );
    apply_color("statusline_error_fg", &mut overrides.statusline_error_fg);

    overrides
}

fn parse_lua_color(value: Result<mlua::BorrowedStr<'_>, mlua::Error>) -> Result<Color, String> {
    let raw = value.map_err(|err| format!("lua theme value is not valid utf-8: {err}"))?;
    let raw = raw.as_ref();
    let normalized = raw.trim();
    let normalized = normalized.strip_prefix('#').unwrap_or(normalized);
    if normalized.len() != 6 {
        return Err(format!(
            "lua theme value '{raw}' must be a 6-digit hex color like #RRGGBB"
        ));
    }

    let r = u8::from_str_radix(&normalized[0..2], 16)
        .map_err(|_| format!("lua theme value '{raw}' contains invalid hex digits"))?;
    let g = u8::from_str_radix(&normalized[2..4], 16)
        .map_err(|_| format!("lua theme value '{raw}' contains invalid hex digits"))?;
    let b = u8::from_str_radix(&normalized[4..6], 16)
        .map_err(|_| format!("lua theme value '{raw}' contains invalid hex digits"))?;

    Ok(Color::Rgb(r, g, b))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use ratatui::style::Color;

    use super::{
        apply_theme_overrides, load_lua_theme_overrides_from_path,
        load_lua_theme_overrides_from_source,
    };
    use crate::settings::ThemePalette;

    #[test]
    fn overrides_apply_only_specified_values() {
        let source = r###"
            theme = {
              tab_active_bg = "#87CEFA",
              statusline_fg = "#FFFFFF",
            }
        "###;

        let overrides = load_lua_theme_overrides_from_source(source).expect("load overrides");
        let base = ThemePalette::default();
        let updated = apply_theme_overrides(base, overrides);

        assert_eq!(updated.tab_active_bg, Color::Rgb(0x87, 0xCE, 0xFA));
        assert_eq!(updated.statusline_fg, Color::Rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(updated.tab_active_fg, base.tab_active_fg);
    }

    #[test]
    fn invalid_values_fall_back_to_defaults() {
        let source = r###"
            theme = {
              statusline_fg = 12,
              tab_active_bg = "#GGGGGG",
            }
        "###;

        let overrides = load_lua_theme_overrides_from_source(source).expect("load overrides");
        let base = ThemePalette::default();
        let updated = apply_theme_overrides(base, overrides);

        assert_eq!(updated.statusline_fg, base.statusline_fg);
        assert_eq!(updated.tab_active_bg, base.tab_active_bg);
    }

    #[test]
    fn missing_theme_table_keeps_defaults() {
        let overrides =
            load_lua_theme_overrides_from_source("print('no theme')").expect("load overrides");
        let base = ThemePalette::default();
        let updated = apply_theme_overrides(base, overrides);

        assert_eq!(updated.tab_inactive_bg, base.tab_inactive_bg);
        assert_eq!(updated.statusline_bg, base.statusline_bg);
    }

    #[test]
    fn load_overrides_from_file_reads_init() {
        let dir = create_temp_dir("yeet-lua-theme");
        let path = dir.join("init.lua");
        let source = r###"
            theme = {
              statusline_dim_fg = "#123456",
            }
        "###;
        fs::write(&path, source).expect("write lua config");

        let overrides = load_lua_theme_overrides_from_path(&path).expect("load overrides");
        let base = ThemePalette::default();
        let updated = apply_theme_overrides(base, overrides);

        assert_eq!(updated.statusline_dim_fg, Color::Rgb(0x12, 0x34, 0x56));
    }

    fn create_temp_dir(label: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let unique = format!(
            "{label}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("timestamp")
                .as_nanos()
        );
        path.push(unique);
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }
}
