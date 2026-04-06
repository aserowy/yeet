mod hook;
mod viewport;

pub use hook::invoke_on_window_create;
pub use mlua::Lua;

pub type LuaConfiguration = Lua;

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

    let hook_mt = create_hook_metatable(lua)?;
    let on_window_create = lua.create_table()?;
    on_window_create.set_metatable(Some(hook_mt));
    hook_table.set("on_window_create", on_window_create)?;

    y_table.set("theme", theme_table)?;
    y_table.set("hook", hook_table)?;
    lua.globals().set("y", y_table)?;

    let content = std::fs::read_to_string(config_path).map_err(LuaError::external)?;
    lua.load(&content)
        .set_name(config_path.to_string_lossy())
        .exec()?;

    Ok(())
}

fn create_hook_metatable(lua: &Lua) -> LuaResult<LuaTable> {
    let methods = lua.create_table()?;
    methods.set(
        "add",
        lua.create_function(|_, (this, func): (LuaTable, LuaValue)| {
            match func {
                LuaValue::Function(_) => {
                    let len = this.raw_len();
                    this.raw_set(len + 1, func)?;
                }
                LuaValue::Nil => {
                    tracing::warn!("y.hook:add() called with nil, ignoring");
                }
                _ => {
                    tracing::warn!(
                        "y.hook:add() called with {}, expected function, ignoring",
                        func.type_name()
                    );
                }
            }
            Ok(())
        })?,
    )?;

    let mt = lua.create_table()?;
    mt.set("__index", methods)?;
    Ok(mt)
}

fn resolve_config_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg).join("yeet").join("init.lua");
        return Some(path);
    }

    dirs::home_dir().map(|home| home.join(".config").join("yeet").join("init.lua"))
}

pub fn read_theme_pairs(lua: &LuaConfiguration) -> Vec<(String, LuaValue)> {
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
        let owc: LuaTable = hook.get("on_window_create").unwrap();
        assert_eq!(owc.raw_len(), 0);
    }

    #[test]
    fn user_can_add_function_to_hook() {
        let lua =
            create_lua_from_script("y.hook.on_window_create:add(function(ctx) return ctx end)");
        let y: LuaTable = lua.globals().get("y").unwrap();
        let hook: LuaTable = y.get("hook").unwrap();
        let owc: LuaTable = hook.get("on_window_create").unwrap();
        assert_eq!(owc.raw_len(), 1);
        let func: LuaValue = owc.raw_get(1).unwrap();
        assert!(matches!(func, LuaValue::Function(_)));
    }

    #[test]
    fn user_can_add_multiple_functions_to_hook() {
        let lua = create_lua_from_script(
            r#"
            y.hook.on_window_create:add(function(ctx) end)
            y.hook.on_window_create:add(function(ctx) end)
            y.hook.on_window_create:add(function(ctx) end)
            "#,
        );
        let y: LuaTable = lua.globals().get("y").unwrap();
        let hook: LuaTable = y.get("hook").unwrap();
        let owc: LuaTable = hook.get("on_window_create").unwrap();
        assert_eq!(owc.raw_len(), 3);
    }

    #[test]
    fn add_non_function_is_ignored() {
        let lua = create_lua_from_script(
            r#"
            y.hook.on_window_create:add("not a function")
            y.hook.on_window_create:add(42)
            y.hook.on_window_create:add(nil)
            "#,
        );
        let y: LuaTable = lua.globals().get("y").unwrap();
        let hook: LuaTable = y.get("hook").unwrap();
        let owc: LuaTable = hook.get("on_window_create").unwrap();
        assert_eq!(owc.raw_len(), 0);
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
