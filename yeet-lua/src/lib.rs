mod hook;
mod loading;
mod plugin;
mod viewport;

pub use hook::invoke_on_bufferline_mutate;
pub use hook::invoke_on_window_create;
pub use hook::BufferType;
pub use loading::load_plugins;
pub use mlua::Lua;
pub use plugin::{read_plugin_concurrency, read_plugin_specs};

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
    let _ = on_window_create.set_metatable(Some(hook_mt.clone()));
    hook_table.set("on_window_create", on_window_create)?;

    let on_bufferline_mutate = lua.create_table()?;
    let _ = on_bufferline_mutate.set_metatable(Some(hook_mt));
    hook_table.set("on_bufferline_mutate", on_bufferline_mutate)?;

    let plugin_table = plugin::create_plugin_table(lua)?;

    if let Some(data_path) = yeet_plugin::resolve_plugin_data_path() {
        plugin_table.set("_data_path", data_path.to_string_lossy().to_string())?;
    }

    y_table.set("theme", theme_table)?;
    y_table.set("hook", hook_table)?;
    y_table.set("plugin", plugin_table)?;

    protect_y_table(lua, y_table)?;
    install_plugin_searcher(lua)?;

    let content = std::fs::read_to_string(config_path).map_err(LuaError::external)?;
    lua.load(&content)
        .set_name(config_path.to_string_lossy())
        .exec()?;

    Ok(())
}

fn protect_y_table(lua: &Lua, y_table: LuaTable) -> LuaResult<()> {
    let globals = lua.globals();
    let g_meta = lua.create_table()?;

    let y_for_index = y_table.clone();
    g_meta.set(
        "__index",
        lua.create_function(move |_, (_t, key): (LuaTable, String)| {
            if key == "y" {
                Ok(LuaValue::Table(y_for_index.clone()))
            } else {
                Ok(LuaValue::Nil)
            }
        })?,
    )?;

    let y_for_newindex = y_table;
    g_meta.set(
        "__newindex",
        lua.create_function(move |_, (t, key, value): (LuaTable, String, LuaValue)| {
            if key == "y" {
                match value {
                    LuaValue::Table(new_table) => {
                        for pair in new_table.pairs::<LuaValue, LuaValue>() {
                            let (k, v) = pair?;
                            y_for_newindex.set(k, v)?;
                        }
                    }
                    _ => {
                        tracing::warn!(
                            "attempt to assign non-table value to y (got {}), ignoring",
                            value.type_name()
                        );
                    }
                }
            } else {
                t.raw_set(key, value)?;
            }
            Ok(())
        })?,
    )?;

    let _ = globals.set_metatable(Some(g_meta));
    Ok(())
}

fn install_plugin_searcher(lua: &Lua) -> LuaResult<()> {
    lua.load(
        r#"
        local noop_mt = {
            __index = function(_, _)
                return function() end
            end
        }

        local function url_to_storage(url)
            url = url:gsub("/$", ""):gsub("%.git$", "")
            url = url:gsub("^https://", ""):gsub("^http://", ""):gsub("^git://", "")
            local parts = {}
            for part in url:gmatch("[^/]+") do
                parts[#parts + 1] = part
            end
            if #parts >= 2 then
                return parts[#parts - 1] .. "/" .. parts[#parts]
            end
        end

        local function plugin_searcher(modname)
            local plugins = y.plugin._plugins
            local data_path = y.plugin._data_path

            for i = 1, #plugins do
                local p = plugins[i]
                local pname = p.name
                if not pname then
                    local url = p.url:gsub("/$", ""):gsub("%.git$", "")
                    pname = url:match("[^/]+$") or url
                end
                if pname == modname then
                    if data_path then
                        local storage = url_to_storage(p.url)
                        if storage then
                            local init_path = data_path .. "/" .. storage .. "/init.lua"
                            local f = io.open(init_path, "r")
                            if f then
                                f:close()
                                return function()
                                    local result = dofile(init_path)
                                    if result ~= nil then
                                        package.loaded[modname] = result
                                    end
                                    return result or setmetatable({}, noop_mt)
                                end
                            end
                        end
                    end
                    return function()
                        return setmetatable({}, noop_mt)
                    end
                end
            end
            return nil
        end

        table.insert(package.searchers, 2, plugin_searcher)
        "#,
    )
    .exec()
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

    #[test]
    fn y_assignment_merges_theme_and_preserves_hook() {
        let lua = create_lua_from_script(
            r##"
            y = { theme = { TabBarActiveBg = "#ff0000" } }
            y.hook.on_window_create:add(function(ctx) end)
            "##,
        );
        let y: LuaTable = lua.globals().get("y").unwrap();

        let theme: LuaTable = y.get("theme").unwrap();
        let val: String = theme.get("TabBarActiveBg").unwrap();
        assert_eq!(val, "#ff0000");

        let hook: LuaTable = y.get("hook").unwrap();
        let owc: LuaTable = hook.get("on_window_create").unwrap();
        assert_eq!(owc.raw_len(), 1);
    }

    #[test]
    fn y_assignment_preserves_hook_add_method() {
        let lua = create_lua_from_script(
            r#"
            y = { theme = { syntax = "base16-ocean.dark" } }
            y.hook.on_window_create:add(function(ctx)
                if ctx.type == "directory" then
                    ctx.preview.wrap = true
                end
            end)
            "#,
        );
        let y: LuaTable = lua.globals().get("y").unwrap();
        let hook: LuaTable = y.get("hook").unwrap();
        let owc: LuaTable = hook.get("on_window_create").unwrap();
        assert_eq!(owc.raw_len(), 1);
    }

    #[test]
    fn y_nil_assignment_does_not_destroy_table() {
        let lua = create_lua_from_script("y = nil");
        let y: LuaTable = lua.globals().get("y").unwrap();
        let hook: LuaTable = y.get("hook").unwrap();
        let _owc: LuaTable = hook.get("on_window_create").unwrap();
    }

    #[test]
    fn other_globals_still_work() {
        let lua = create_lua_from_script("foo = 42");
        let val: i64 = lua.globals().get("foo").unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn plugin_table_exists_after_init() {
        let lua = create_lua_from_script("");
        let y: LuaTable = lua.globals().get("y").unwrap();
        let plugin: LuaTable = y.get("plugin").unwrap();
        let plugins: LuaTable = plugin.get("_plugins").unwrap();
        assert_eq!(plugins.raw_len(), 0);
    }

    #[test]
    fn register_plugin_with_all_opts() {
        let lua = create_lua_from_script(
            r#"
            y.plugin.register({
                url = "https://github.com/user/yeet-nord",
                branch = "main",
                version = ">=1.0, <2.0"
            })
            "#,
        );
        let specs = read_plugin_specs(&lua);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].url, "https://github.com/user/yeet-nord");
        assert_eq!(specs[0].branch.as_deref(), Some("main"));
        assert_eq!(specs[0].version.as_deref(), Some(">=1.0, <2.0"));
    }

    #[test]
    fn register_plugin_url_only() {
        let lua = create_lua_from_script(
            r#"y.plugin.register({ url = "https://github.com/user/plugin" })"#,
        );
        let specs = read_plugin_specs(&lua);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].url, "https://github.com/user/plugin");
        assert!(specs[0].branch.is_none());
        assert!(specs[0].version.is_none());
    }

    #[test]
    fn register_plugin_with_dependencies() {
        let lua = create_lua_from_script(
            r#"
            y.plugin.register({
                url = "https://github.com/user/theme",
                dependencies = {
                    { url = "https://github.com/user/lib", version = ">=0.5" }
                }
            })
            "#,
        );
        let specs = read_plugin_specs(&lua);
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].dependencies.len(), 1);
        assert_eq!(specs[0].dependencies[0].url, "https://github.com/user/lib");
        assert_eq!(specs[0].dependencies[0].version.as_deref(), Some(">=0.5"));
    }

    #[test]
    fn register_without_url_is_ignored() {
        let lua = create_lua_from_script(r#"y.plugin.register({ branch = "main" })"#);
        let specs = read_plugin_specs(&lua);
        assert!(specs.is_empty());
    }

    #[test]
    fn register_with_non_table_is_ignored() {
        let lua = create_lua_from_script(r#"y.plugin.register("https://github.com/user/plugin")"#);
        let specs = read_plugin_specs(&lua);
        assert!(specs.is_empty());
    }

    #[test]
    fn concurrency_default() {
        let lua = create_lua_from_script("");
        assert_eq!(read_plugin_concurrency(&lua), 4);
    }

    #[test]
    fn concurrency_custom() {
        let lua = create_lua_from_script("y.plugin.concurrency = 2");
        assert_eq!(read_plugin_concurrency(&lua), 2);
    }

    #[test]
    fn concurrency_invalid_uses_default() {
        let lua = create_lua_from_script(r#"y.plugin.concurrency = "fast""#);
        assert_eq!(read_plugin_concurrency(&lua), 4);
    }

    #[test]
    fn plugin_survives_y_reassignment() {
        let lua = create_lua_from_script(
            r##"
            y = { theme = { TabBarActiveBg = "#ff0000" } }
            y.plugin.register({ url = "https://github.com/user/plugin" })
            "##,
        );
        let specs = read_plugin_specs(&lua);
        assert_eq!(specs.len(), 1);

        let y: LuaTable = lua.globals().get("y").unwrap();
        let theme: LuaTable = y.get("theme").unwrap();
        let val: String = theme.get("TabBarActiveBg").unwrap();
        assert_eq!(val, "#ff0000");
    }

    #[test]
    fn multiple_plugins_registered() {
        let lua = create_lua_from_script(
            r#"
            y.plugin.register({ url = "https://github.com/a/one" })
            y.plugin.register({ url = "https://github.com/b/two" })
            "#,
        );
        let specs = read_plugin_specs(&lua);
        assert_eq!(specs.len(), 2);
        assert_eq!(specs[0].url, "https://github.com/a/one");
        assert_eq!(specs[1].url, "https://github.com/b/two");
    }

    #[test]
    fn require_unloaded_plugin_returns_noop_proxy() {
        let lua = create_lua_from_script(
            r#"
            y.plugin.register({ url = "https://github.com/user/yeet-theme", name = "my-theme" })
            require('my-theme').setup()
            "#,
        );
        let specs = read_plugin_specs(&lua);
        assert_eq!(specs.len(), 1);
    }

    #[test]
    fn require_unknown_module_still_errors() {
        let lua = Lua::new();
        let mut tmp = NamedTempFile::new().unwrap();
        write!(
            tmp,
            r#"
            y.plugin.register({{ url = "https://github.com/user/yeet-theme" }})
            require('totally-unknown-module').setup()
            "#
        )
        .unwrap();
        let path = tmp.path().to_path_buf();
        let result = setup_and_execute(&lua, &path);
        assert!(result.is_err());
    }

    #[test]
    fn register_ssh_url_is_rejected() {
        let lua = create_lua_from_script(
            r#"y.plugin.register({ url = "git@github.com:user/repo.git" })"#,
        );
        let specs = read_plugin_specs(&lua);
        assert!(specs.is_empty());
    }

    #[test]
    fn register_http_url_is_rejected() {
        let lua =
            create_lua_from_script(r#"y.plugin.register({ url = "http://github.com/user/repo" })"#);
        let specs = read_plugin_specs(&lua);
        assert!(specs.is_empty());
    }

    #[test]
    fn register_https_url_succeeds() {
        let lua = create_lua_from_script(
            r#"y.plugin.register({ url = "https://github.com/user/repo" })"#,
        );
        let specs = read_plugin_specs(&lua);
        assert_eq!(specs.len(), 1);
    }
}
