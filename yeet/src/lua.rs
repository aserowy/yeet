use std::path::PathBuf;

use mlua::prelude::*;
use yeet_frontend::theme::{parse_hex_color, Theme};

pub fn load_theme() -> Theme {
    let mut theme = Theme::default();

    let config_path = resolve_config_path();
    let config_path = match config_path {
        Some(path) => path,
        None => {
            tracing::info!("no config directory found, using default theme");
            return theme;
        }
    };

    if !config_path.exists() {
        tracing::info!("no init.lua found at {:?}, using default theme", config_path);
        return theme;
    }

    let lua = match Lua::new() {
        lua => lua,
    };

    if let Err(err) = run_init_lua(&lua, &config_path, &mut theme) {
        tracing::error!("error loading init.lua: {:?}", err);
    }

    theme
}

fn run_init_lua(lua: &Lua, config_path: &PathBuf, theme: &mut Theme) -> LuaResult<()> {
    // Create y.theme table
    let y_table = lua.create_table()?;
    let theme_table = lua.create_table()?;
    y_table.set("theme", theme_table)?;
    lua.globals().set("y", y_table)?;

    // Load and execute init.lua
    let content = std::fs::read_to_string(config_path).map_err(LuaError::external)?;
    lua.load(&content)
        .set_name(config_path.to_string_lossy())
        .exec()?;

    // Read theme values
    let y: LuaTable = lua.globals().get("y")?;
    let theme_table: LuaTable = y.get("theme")?;

    for pair in theme_table.pairs::<String, LuaValue>() {
        let (key, value) = pair?;

        if key == "syntax" {
            if let LuaValue::String(s) = value {
                theme.syntax_theme = s.to_str()?.to_string();
                tracing::info!("syntax theme set to: {}", theme.syntax_theme);
            }
            continue;
        }

        if let LuaValue::String(s) = value {
            let hex = s.to_str()?;
            match parse_hex_color(&hex) {
                Some(color) => {
                    tracing::info!("theme token '{}' set to '{}'", key, hex);
                    theme.set_color(key, color);
                }
                None => {
                    tracing::error!("invalid color value '{}' for token '{}', using default", hex, key);
                }
            }
        }
    }

    Ok(())
}

fn resolve_config_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg).join("yeet").join("init.lua");
        return Some(path);
    }

    dirs::home_dir().map(|home| home.join(".config").join("yeet").join("init.lua"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn load_theme_from_script(script: &str) -> Theme {
        let mut theme = Theme::default();
        let lua = Lua::new();

        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{}", script).unwrap();
        let path = tmp.path().to_path_buf();

        let _ = run_init_lua(&lua, &path, &mut theme);
        theme
    }

    #[test]
    fn lua_sets_theme_color() {
        use ratatui::style::Color;
        let theme = load_theme_from_script("y.theme.TabBarActiveBg = '#ff0000'");
        assert_eq!(
            theme.color(yeet_frontend::theme::tokens::TABBAR_ACTIVE_BG),
            Color::Rgb(255, 0, 0)
        );
    }

    #[test]
    fn lua_sets_syntax_theme() {
        let theme = load_theme_from_script("y.theme.syntax = 'base16-ocean.dark'");
        assert_eq!(theme.syntax_theme, "base16-ocean.dark");
    }

    #[test]
    fn lua_invalid_color_keeps_default() {
        let default_theme = Theme::default();
        let theme = load_theme_from_script("y.theme.TabBarActiveBg = 'not-a-color'");
        assert_eq!(
            theme.color(yeet_frontend::theme::tokens::TABBAR_ACTIVE_BG),
            default_theme.color(yeet_frontend::theme::tokens::TABBAR_ACTIVE_BG)
        );
    }

    #[test]
    fn lua_syntax_error_returns_defaults() {
        let default_theme = Theme::default();
        let theme = load_theme_from_script("this is not valid lua!!!");
        assert_eq!(theme.syntax_theme, default_theme.syntax_theme);
    }
}
