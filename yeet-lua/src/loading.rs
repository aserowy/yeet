use std::collections::HashSet;
use std::path::Path;

use mlua::prelude::*;
use yeet_plugin::{PluginSpec, PluginState, PluginStatus};

use crate::plugin::read_plugin_specs;

pub fn load_plugins(lua: &Lua, data_path: &Path) -> Vec<PluginState> {
    let specs = read_plugin_specs(lua);
    if specs.is_empty() {
        return Vec::new();
    }

    let ordered = compute_load_order(&specs);
    let mut states = Vec::new();
    let mut loaded: HashSet<String> = HashSet::new();
    let mut failed: HashSet<String> = HashSet::new();

    for spec in &ordered {
        if loaded.contains(&spec.url) {
            continue;
        }

        if is_dep_failed(spec, &failed, &specs) {
            let msg = format!("dependency failed to load for {}", spec.url);
            states.push(PluginState {
                url: spec.url.clone(),
                status: PluginStatus::Error,
                error_message: Some(msg),
                commit: None,
            });
            failed.insert(spec.url.clone());
            continue;
        }

        let storage_path = yeet_plugin::url_to_storage_path(&spec.url);
        let plugin_dir = match storage_path {
            Some(rel) => data_path.join(rel),
            None => {
                states.push(PluginState {
                    url: spec.url.clone(),
                    status: PluginStatus::Error,
                    error_message: Some(format!("invalid URL: {}", spec.url)),
                    commit: None,
                });
                failed.insert(spec.url.clone());
                continue;
            }
        };

        if !plugin_dir.exists() {
            states.push(PluginState {
                url: spec.url.clone(),
                status: PluginStatus::Missing,
                error_message: Some(format!("directory not found: {}", plugin_dir.display())),
                commit: None,
            });
            failed.insert(spec.url.clone());
            continue;
        }

        let init_path = plugin_dir.join("init.lua");
        if !init_path.exists() {
            states.push(PluginState {
                url: spec.url.clone(),
                status: PluginStatus::Error,
                error_message: Some(format!("no init.lua found in {}", plugin_dir.display())),
                commit: None,
            });
            failed.insert(spec.url.clone());
            continue;
        }

        let plugin_name = spec
            .name
            .clone()
            .unwrap_or_else(|| derive_plugin_name(&spec.url));

        if is_already_loaded(lua, &plugin_name) {
            loaded.insert(spec.url.clone());
            states.push(PluginState {
                url: spec.url.clone(),
                status: PluginStatus::Loaded,
                error_message: None,
                commit: None,
            });
            continue;
        }

        match load_single_plugin(lua, &init_path, &plugin_name) {
            Ok(()) => {
                loaded.insert(spec.url.clone());
                states.push(PluginState {
                    url: spec.url.clone(),
                    status: PluginStatus::Loaded,
                    error_message: None,
                    commit: None,
                });
            }
            Err(err) => {
                failed.insert(spec.url.clone());
                states.push(PluginState {
                    url: spec.url.clone(),
                    status: PluginStatus::Error,
                    error_message: Some(err.to_string()),
                    commit: None,
                });
            }
        }
    }

    report_missing_plugins(&states);

    states
}

fn is_already_loaded(lua: &Lua, plugin_name: &str) -> bool {
    let Ok(package) = lua.globals().get::<LuaTable>("package") else {
        return false;
    };
    let Ok(loaded) = package.get::<LuaTable>("loaded") else {
        return false;
    };
    !matches!(
        loaded.get::<LuaValue>(plugin_name),
        Ok(LuaValue::Nil) | Err(_)
    )
}

fn load_single_plugin(lua: &Lua, init_path: &Path, plugin_name: &str) -> LuaResult<()> {
    let snapshot = take_snapshot(lua)?;

    let plugin_dir = init_path
        .parent()
        .ok_or_else(|| LuaError::external("plugin init.lua has no parent directory"))?;

    prepend_package_path(lua, plugin_dir)?;

    let content = std::fs::read_to_string(init_path).map_err(LuaError::external)?;
    let result: LuaResult<LuaValue> = lua
        .load(&content)
        .set_name(init_path.to_string_lossy())
        .eval();

    match result {
        Ok(value) => {
            if let LuaValue::Table(_) = &value {
                let loaded: LuaTable = lua.globals().get::<LuaTable>("package")?.get("loaded")?;
                loaded.set(plugin_name, value)?;
            }
            Ok(())
        }
        Err(err) => {
            tracing::error!("plugin {} failed: {}", init_path.display(), err);
            restore_snapshot(lua, snapshot)?;
            Err(err)
        }
    }
}

fn prepend_package_path(lua: &Lua, plugin_dir: &Path) -> LuaResult<()> {
    let package: LuaTable = lua.globals().get("package")?;
    let current_path: String = package.get("path")?;
    let dir = plugin_dir.to_string_lossy();
    let new_path = format!("{}/?.lua;{}/?/init.lua;{}", dir, dir, current_path);
    package.set("path", new_path)?;
    Ok(())
}

pub fn derive_plugin_name(url: &str) -> String {
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    url.rsplit('/').next().unwrap_or(url).to_string()
}

fn shallow_clone_table(lua: &Lua, source: &LuaTable) -> LuaResult<LuaTable> {
    let clone = lua.create_table()?;
    for pair in source.pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair?;
        clone.set(k, v)?;
    }
    Ok(clone)
}

fn restore_table_from_clone(target: &LuaTable, source: &LuaTable) -> LuaResult<()> {
    let keys_to_clear: Vec<LuaValue> = target
        .pairs::<LuaValue, LuaValue>()
        .filter_map(|r| r.ok())
        .map(|(k, _)| k)
        .collect();

    for key in keys_to_clear {
        target.set(key, LuaValue::Nil)?;
    }

    for pair in source.pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair?;
        target.set(k, v)?;
    }

    Ok(())
}

fn take_snapshot(lua: &Lua) -> LuaResult<PluginSnapshot> {
    let y: LuaTable = lua.globals().get("y")?;

    let hook: LuaTable = y.get("hook")?;
    let on_window_create: LuaTable = hook.get("on_window_create")?;
    let hook_on_window_create = shallow_clone_table(lua, &on_window_create)?;

    let on_bufferline_mutate: LuaTable = hook.get("on_bufferline_mutate")?;
    let hook_on_bufferline_mutate = shallow_clone_table(lua, &on_bufferline_mutate)?;

    let theme: LuaTable = y.get("theme")?;
    let theme_clone = shallow_clone_table(lua, &theme)?;

    Ok(PluginSnapshot {
        hook_on_window_create,
        hook_on_bufferline_mutate,
        theme: theme_clone,
    })
}

fn restore_snapshot(lua: &Lua, snapshot: PluginSnapshot) -> LuaResult<()> {
    let y: LuaTable = lua.globals().get("y")?;

    let hook: LuaTable = y.get("hook")?;
    let on_window_create: LuaTable = hook.get("on_window_create")?;
    restore_table_from_clone(&on_window_create, &snapshot.hook_on_window_create)?;

    let on_bufferline_mutate: LuaTable = hook.get("on_bufferline_mutate")?;
    restore_table_from_clone(&on_bufferline_mutate, &snapshot.hook_on_bufferline_mutate)?;

    let theme: LuaTable = y.get("theme")?;
    restore_table_from_clone(&theme, &snapshot.theme)?;

    Ok(())
}

struct PluginSnapshot {
    hook_on_window_create: LuaTable,
    hook_on_bufferline_mutate: LuaTable,
    theme: LuaTable,
}

fn compute_load_order(specs: &[PluginSpec]) -> Vec<&PluginSpec> {
    let mut order = Vec::new();
    let mut seen = HashSet::new();

    for spec in specs {
        for dep in &spec.dependencies {
            if seen.insert(dep.url.clone()) {
                order.push(dep);
            }
        }
        if seen.insert(spec.url.clone()) {
            order.push(spec);
        }
    }

    order
}

fn is_dep_failed(spec: &PluginSpec, failed: &HashSet<String>, all_specs: &[PluginSpec]) -> bool {
    for parent in all_specs {
        for dep in &parent.dependencies {
            if dep.url == spec.url {
                continue;
            }
        }
        if parent.url == spec.url {
            for dep in &parent.dependencies {
                if failed.contains(&dep.url) {
                    return true;
                }
            }
        }
    }
    false
}

fn report_missing_plugins(states: &[PluginState]) {
    let missing: Vec<&str> = states
        .iter()
        .filter(|s| s.status == PluginStatus::Missing)
        .map(|s| s.url.as_str())
        .collect();

    if !missing.is_empty() {
        tracing::error!(
            "missing plugins (run :pluginsync or :pluginupdate): {}",
            missing.join(", ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as IoWrite;
    use tempfile::TempDir;

    fn setup_lua() -> Lua {
        let lua = Lua::new();
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        write!(tmp, "").unwrap();
        let path = tmp.path().to_path_buf();
        crate::setup_and_execute(&lua, &path).unwrap();
        lua
    }

    #[test]
    fn no_plugins_returns_empty() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();
        let states = load_plugins(&lua, dir.path());
        assert!(states.is_empty());
    }

    #[test]
    fn missing_plugin_recorded() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/missing" })"#)
            .exec()
            .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].status, PluginStatus::Missing);
    }

    #[test]
    fn successful_load() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let plugin_dir = dir.path().join("test").join("plugin");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(plugin_dir.join("init.lua"), "y.theme.TestColor = '#123456'").unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/plugin" })"#)
            .exec()
            .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].status, PluginStatus::Loaded);

        let y: LuaTable = lua.globals().get("y").unwrap();
        let theme: LuaTable = y.get("theme").unwrap();
        let val: String = theme.get("TestColor").unwrap();
        assert_eq!(val, "#123456");
    }

    #[test]
    fn syntax_error_rollback() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let plugin_dir = dir.path().join("test").join("broken");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("init.lua"),
            r#"
            y.theme.BrokenColor = '#000000'
            this is not valid lua!!!
            "#,
        )
        .unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/broken" })"#)
            .exec()
            .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].status, PluginStatus::Error);

        let y: LuaTable = lua.globals().get("y").unwrap();
        let theme: LuaTable = y.get("theme").unwrap();
        let val: LuaValue = theme.get("BrokenColor").unwrap();
        assert!(matches!(val, LuaValue::Nil));
    }

    #[test]
    fn runtime_error_rollback_hooks() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let plugin_dir = dir.path().join("test").join("hookfail");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("init.lua"),
            r#"
            y.hook.on_window_create:add(function(ctx) end)
            error("intentional failure")
            "#,
        )
        .unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/hookfail" })"#)
            .exec()
            .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states[0].status, PluginStatus::Error);

        let y: LuaTable = lua.globals().get("y").unwrap();
        let hook: LuaTable = y.get("hook").unwrap();
        let owc: LuaTable = hook.get("on_window_create").unwrap();
        assert_eq!(owc.raw_len(), 0);
    }

    #[test]
    fn earlier_plugin_hooks_survive_later_failure() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let good_dir = dir.path().join("test").join("good");
        std::fs::create_dir_all(&good_dir).unwrap();
        std::fs::write(
            good_dir.join("init.lua"),
            "y.hook.on_window_create:add(function(ctx) end)",
        )
        .unwrap();

        let bad_dir = dir.path().join("test").join("bad");
        std::fs::create_dir_all(&bad_dir).unwrap();
        std::fs::write(
            bad_dir.join("init.lua"),
            r#"
            y.hook.on_window_create:add(function(ctx) end)
            error("intentional failure")
            "#,
        )
        .unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/good" })"#)
            .exec()
            .unwrap();
        lua.load(r#"y.plugin.register({ url = "https://github.com/test/bad" })"#)
            .exec()
            .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states[0].status, PluginStatus::Loaded);
        assert_eq!(states[1].status, PluginStatus::Error);

        let y: LuaTable = lua.globals().get("y").unwrap();
        let hook: LuaTable = y.get("hook").unwrap();
        let owc: LuaTable = hook.get("on_window_create").unwrap();
        assert_eq!(owc.raw_len(), 1);
        let func: LuaValue = owc.raw_get(1).unwrap();
        assert!(matches!(func, LuaValue::Function(_)));
    }

    #[test]
    fn earlier_plugin_theme_survives_later_failure() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let good_dir = dir.path().join("test").join("themer");
        std::fs::create_dir_all(&good_dir).unwrap();
        std::fs::write(good_dir.join("init.lua"), "y.theme.GoodColor = '#aabbcc'").unwrap();

        let bad_dir = dir.path().join("test").join("overrider");
        std::fs::create_dir_all(&bad_dir).unwrap();
        std::fs::write(
            bad_dir.join("init.lua"),
            r#"
            y.theme.GoodColor = '#000000'
            y.theme.BadColor = '#ffffff'
            error("intentional failure")
            "#,
        )
        .unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/themer" })"#)
            .exec()
            .unwrap();
        lua.load(r#"y.plugin.register({ url = "https://github.com/test/overrider" })"#)
            .exec()
            .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states[0].status, PluginStatus::Loaded);
        assert_eq!(states[1].status, PluginStatus::Error);

        let y: LuaTable = lua.globals().get("y").unwrap();
        let theme: LuaTable = y.get("theme").unwrap();
        let good: String = theme.get("GoodColor").unwrap();
        assert_eq!(good, "#aabbcc");
        let bad: LuaValue = theme.get("BadColor").unwrap();
        assert!(matches!(bad, LuaValue::Nil));
    }

    #[test]
    fn missing_init_lua() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let plugin_dir = dir.path().join("test").join("noinit");
        std::fs::create_dir_all(&plugin_dir).unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/noinit" })"#)
            .exec()
            .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].status, PluginStatus::Error);
        assert!(states[0]
            .error_message
            .as_ref()
            .unwrap()
            .contains("no init.lua"));
    }

    #[test]
    fn derive_plugin_name_uses_full_segment() {
        assert_eq!(
            derive_plugin_name("https://github.com/aserowy/yeet-bluloco-theme"),
            "yeet-bluloco-theme"
        );
    }

    #[test]
    fn derive_plugin_name_no_prefix() {
        assert_eq!(
            derive_plugin_name("https://github.com/user/cool-plugin"),
            "cool-plugin"
        );
    }

    #[test]
    fn derive_plugin_name_with_git_suffix() {
        assert_eq!(
            derive_plugin_name("https://github.com/user/yeet-theme.git"),
            "yeet-theme"
        );
    }

    #[test]
    fn plugin_module_available_via_require() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let plugin_dir = dir.path().join("test").join("yeet-my-mod");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("init.lua"),
            r#"
            local M = {}
            function M.greet() return "hello" end
            return M
            "#,
        )
        .unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/yeet-my-mod" })"#)
            .exec()
            .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states[0].status, PluginStatus::Loaded);

        let result: String = lua
            .load(r#"return require('yeet-my-mod').greet()"#)
            .eval()
            .unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn explicit_name_overrides_url_derived_name() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let plugin_dir = dir.path().join("test").join("yeet-my-mod");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("init.lua"),
            r#"
            local M = {}
            function M.greet() return "custom" end
            return M
            "#,
        )
        .unwrap();

        lua.load(
            r#"y.plugin.register({ url = "https://github.com/test/yeet-my-mod", name = "my-mod" })"#,
        )
        .exec()
        .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states[0].status, PluginStatus::Loaded);

        let result: String = lua
            .load(r#"return require('my-mod').greet()"#)
            .eval()
            .unwrap();
        assert_eq!(result, "custom");
    }

    #[test]
    fn plugin_returning_nil_still_loads() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let plugin_dir = dir.path().join("test").join("yeet-no-return");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(plugin_dir.join("init.lua"), "y.theme.SomeColor = '#112233'").unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/yeet-no-return" })"#)
            .exec()
            .unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states[0].status, PluginStatus::Loaded);

        let y: LuaTable = lua.globals().get("y").unwrap();
        let theme: LuaTable = y.get("theme").unwrap();
        let val: String = theme.get("SomeColor").unwrap();
        assert_eq!(val, "#112233");
    }

    #[test]
    fn require_loads_from_disk_and_setup_persists() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let plugin_dir = dir.path().join("test").join("yeet-my-theme");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("init.lua"),
            r#"
            local M = {}
            function M.setup()
                y.theme.MyThemeColor = '#aabbcc'
            end
            return M
            "#,
        )
        .unwrap();

        let data_path_str = dir.path().to_string_lossy().to_string();
        let y: LuaTable = lua.globals().get("y").unwrap();
        let plugin: LuaTable = y.get("plugin").unwrap();
        plugin.set("_data_path", data_path_str).unwrap();

        lua.load(
            r#"
            y.plugin.register({ url = "https://github.com/test/yeet-my-theme" })
            require('yeet-my-theme').setup()
            "#,
        )
        .exec()
        .unwrap();

        let theme: LuaTable = y.get("theme").unwrap();
        let val: String = theme.get("MyThemeColor").unwrap();
        assert_eq!(val, "#aabbcc");
    }

    #[test]
    fn require_returns_proxy_when_not_on_disk() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let data_path_str = dir.path().to_string_lossy().to_string();
        let y: LuaTable = lua.globals().get("y").unwrap();
        let plugin: LuaTable = y.get("plugin").unwrap();
        plugin.set("_data_path", data_path_str).unwrap();

        lua.load(
            r#"
            y.plugin.register({ url = "https://github.com/test/yeet-missing" })
            require('yeet-missing').setup()
            "#,
        )
        .exec()
        .unwrap();

        let specs = crate::read_plugin_specs(&lua);
        assert_eq!(specs.len(), 1);
    }

    #[test]
    fn plugin_loaded_via_require_not_double_loaded() {
        let lua = setup_lua();
        let dir = TempDir::new().unwrap();

        let plugin_dir = dir.path().join("test").join("yeet-counter");
        std::fs::create_dir_all(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("init.lua"),
            r#"
            local M = {}
            M.count = (y.theme.LoadCount or 0) + 1
            y.theme.LoadCount = tostring(M.count)
            return M
            "#,
        )
        .unwrap();

        let data_path_str = dir.path().to_string_lossy().to_string();
        let y: LuaTable = lua.globals().get("y").unwrap();
        let plugin: LuaTable = y.get("plugin").unwrap();
        plugin.set("_data_path", data_path_str).unwrap();

        lua.load(r#"y.plugin.register({ url = "https://github.com/test/yeet-counter" })"#)
            .exec()
            .unwrap();

        lua.load(r#"require('yeet-counter')"#).exec().unwrap();

        let states = load_plugins(&lua, dir.path());
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].status, PluginStatus::Loaded);

        let theme: LuaTable = y.get("theme").unwrap();
        let count: String = theme.get("LoadCount").unwrap();
        assert_eq!(count, "1");
    }
}
