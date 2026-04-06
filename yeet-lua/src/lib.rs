mod hook;
mod viewport;

pub use hook::invoke_on_window_create;
pub use mlua::Lua;

use std::path::PathBuf;

use mlua::prelude::*;

pub fn init() -> Option<Lua> {
    let config_path = resolve_config_path();
    let config_path = match config_path {
        Some(path) => path,
        None => {
            tracing::info!("no config directory found");
            return None;
        }
    };

    if !config_path.exists() {
        tracing::info!("no init.lua found at {:?}", config_path);
        return None;
    }

    let lua = Lua::new();
    match setup_and_execute(&lua, &config_path) {
        Ok(()) => Some(lua),
        Err(err) => {
            tracing::error!("error loading init.lua: {:?}", err);
            None
        }
    }
}

fn setup_and_execute(lua: &Lua, config_path: &PathBuf) -> LuaResult<()> {
    let y_table = lua.create_table()?;
    let theme_table = lua.create_table()?;
    let hook_table = lua.create_table()?;
    y_table.set("theme", theme_table)?;
    y_table.set("hook", hook_table)?;
    lua.globals().set("y", y_table)?;

    let content = std::fs::read_to_string(config_path).map_err(LuaError::external)?;
    lua.load(&content)
        .set_name(config_path.to_string_lossy())
        .exec()?;

    Ok(())
}

fn resolve_config_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg).join("yeet").join("init.lua");
        return Some(path);
    }

    dirs::home_dir().map(|home| home.join(".config").join("yeet").join("init.lua"))
}

pub fn read_theme_pairs(lua: &Lua) -> Vec<(String, LuaValue)> {
    let mut pairs = Vec::new();
    let Ok(y) = lua.globals().get::<LuaTable>("y") else {
        return pairs;
    };
    let Ok(theme_table) = y.get::<LuaTable>("theme") else {
        return pairs;
    };
    for (key, value) in theme_table.pairs::<String, LuaValue>().flatten() {
        pairs.push((key, value));
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_lua_from_script(script: &str) -> Lua {
        let lua = Lua::new();
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{}", script).unwrap();
        let path = tmp.path().to_path_buf();
        setup_and_execute(&lua, &path).unwrap();
        lua
    }

    #[test]
    fn hook_table_exists_after_init() {
        let lua = create_lua_from_script("");
        let y: LuaTable = lua.globals().get("y").unwrap();
        let hook: LuaTable = y.get("hook").unwrap();
        assert_eq!(hook.len().unwrap(), 0);
    }

    #[test]
    fn user_can_assign_function_to_hook() {
        let lua = create_lua_from_script("y.hook.on_window_create = function(ctx) return ctx end");
        let y: LuaTable = lua.globals().get("y").unwrap();
        let hook: LuaTable = y.get("hook").unwrap();
        let func: LuaValue = hook.get("on_window_create").unwrap();
        assert!(matches!(func, LuaValue::Function(_)));
    }

    #[test]
    fn theme_table_exists_after_init() {
        let lua = create_lua_from_script("");
        let y: LuaTable = lua.globals().get("y").unwrap();
        let theme: LuaTable = y.get("theme").unwrap();
        assert_eq!(theme.len().unwrap(), 0);
    }

    #[test]
    fn read_theme_pairs_returns_values() {
        let lua = create_lua_from_script("y.theme.TabBarActiveBg = '#ff0000'");
        let pairs = read_theme_pairs(&lua);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "TabBarActiveBg");
    }

    #[test]
    fn syntax_error_returns_none() {
        let lua = Lua::new();
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "this is not valid lua!!!").unwrap();
        let path = tmp.path().to_path_buf();
        let result = setup_and_execute(&lua, &path);
        assert!(result.is_err());
    }
}
