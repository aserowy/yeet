use mlua::prelude::*;
use yeet_frontend::theme::{parse_hex_color, Theme};
use yeet_lua::LuaConfiguration;
use yeet_plugin::PluginState;

pub struct LuaInit {
    pub theme: Theme,
    pub lua: Option<LuaConfiguration>,
    pub plugin_states: Vec<PluginState>,
    pub plugin_concurrency: usize,
}

pub fn init() -> LuaInit {
    let mut theme = Theme::default();

    let lua = match yeet_lua::init() {
        Some(lua) => lua,
        None => {
            return LuaInit {
                theme,
                lua: None,
                plugin_states: Vec::new(),
                plugin_concurrency: 4,
            };
        }
    };

    read_theme_values(&lua, &mut theme);

    let plugin_concurrency = yeet_lua::read_plugin_concurrency(&lua);

    let plugin_states = match yeet_plugin::resolve_plugin_data_path() {
        Some(data_path) => yeet_lua::load_plugins(&lua, &data_path),
        None => {
            tracing::warn!("could not resolve plugin data path, skipping plugin loading");
            Vec::new()
        }
    };

    read_theme_values(&lua, &mut theme);

    LuaInit {
        theme,
        lua: Some(lua),
        plugin_states,
        plugin_concurrency,
    }
}

fn read_theme_values(lua: &LuaConfiguration, theme: &mut Theme) {
    for (key, value) in yeet_lua::read_theme_pairs(lua) {
        if key == "syntax" {
            if let LuaValue::String(s) = value {
                if let Ok(s) = s.to_str() {
                    theme.syntax_theme = s.to_string();
                    tracing::info!("syntax theme set to: {}", theme.syntax_theme);
                }
            }
            continue;
        }

        if let LuaValue::String(s) = value {
            if let Ok(hex) = s.to_str() {
                match parse_hex_color(&hex) {
                    Some(color) => {
                        tracing::info!("theme token '{}' set to '{}'", key, hex);
                        theme.set_color(key, color);
                    }
                    None => {
                        tracing::error!(
                            "invalid color value '{}' for token '{}', using default",
                            hex,
                            key
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use mlua::prelude::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use yeet_frontend::theme::{parse_hex_color, Theme};

    fn load_theme_from_script(script: &str) -> Theme {
        let mut theme = Theme::default();
        let lua = mlua::Lua::new();

        let y_table = lua.create_table().unwrap();
        let theme_table = lua.create_table().unwrap();
        let hook_table = lua.create_table().unwrap();
        y_table.set("theme", theme_table).unwrap();
        y_table.set("hook", hook_table).unwrap();
        lua.globals().set("y", y_table).unwrap();

        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{}", script).unwrap();
        let path = tmp.path().to_path_buf();
        let content = std::fs::read_to_string(&path).unwrap();
        lua.load(&content).exec().unwrap();

        for (key, value) in yeet_lua::read_theme_pairs(&lua) {
            if key == "syntax" {
                if let LuaValue::String(s) = value {
                    theme.syntax_theme = s.to_str().unwrap().to_string();
                }
                continue;
            }
            if let LuaValue::String(s) = value {
                let hex = s.to_str().unwrap();
                if let Some(color) = parse_hex_color(&hex) {
                    theme.set_color(key, color);
                }
            }
        }

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
    fn lua_overrides_buffer_fg_token() {
        use ratatui::style::Color;
        let theme = load_theme_from_script("y.theme.BufferFg = '#00ff00'");
        assert_eq!(
            theme.color(yeet_frontend::theme::tokens::BUFFER_FG),
            Color::Rgb(0, 255, 0)
        );
    }

    #[test]
    fn lua_plugin_defined_icon_token() {
        use ratatui::style::Color;
        // Simulates a plugin setting a custom icon class token
        let theme = load_theme_from_script("y.theme.IconRust = '#dea584'");
        assert_eq!(theme.color("IconRust"), Color::Rgb(222, 165, 132));
    }

    #[test]
    fn lua_unknown_icon_token_uses_reset_fallback() {
        let theme = load_theme_from_script("");
        assert_eq!(
            theme.color("IconUnmappedClass"),
            ratatui::style::Color::Reset
        );
    }
}
