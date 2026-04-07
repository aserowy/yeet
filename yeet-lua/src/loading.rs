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

    for (spec, is_dependency) in &ordered {
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

        match load_single_plugin(lua, &init_path, *is_dependency) {
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

fn load_single_plugin(lua: &Lua, init_path: &Path, _is_dependency: bool) -> LuaResult<()> {
    let snapshot = take_snapshot(lua)?;

    let content = std::fs::read_to_string(init_path).map_err(LuaError::external)?;
    let result = lua
        .load(&content)
        .set_name(init_path.to_string_lossy())
        .exec();

    if let Err(ref err) = result {
        tracing::error!("plugin {} failed: {}", init_path.display(), err);
        restore_snapshot(lua, snapshot)?;
    }

    result
}

fn take_snapshot(lua: &Lua) -> LuaResult<PluginSnapshot> {
    let y: LuaTable = lua.globals().get("y")?;

    let hook: LuaTable = y.get("hook")?;
    let on_window_create: LuaTable = hook.get("on_window_create")?;
    let hook_count = on_window_create.raw_len();

    let theme: LuaTable = y.get("theme")?;
    let theme_keys: Vec<String> = theme
        .pairs::<String, LuaValue>()
        .filter_map(|r| r.ok())
        .map(|(k, _)| k)
        .collect();

    Ok(PluginSnapshot {
        hook_count,
        theme_keys,
    })
}

fn restore_snapshot(lua: &Lua, snapshot: PluginSnapshot) -> LuaResult<()> {
    let y: LuaTable = lua.globals().get("y")?;

    let hook: LuaTable = y.get("hook")?;
    let on_window_create: LuaTable = hook.get("on_window_create")?;
    let current_len = on_window_create.raw_len();
    for i in (snapshot.hook_count + 1..=current_len).rev() {
        on_window_create.raw_set(i, LuaValue::Nil)?;
    }

    let theme: LuaTable = y.get("theme")?;
    let current_keys: Vec<String> = theme
        .pairs::<String, LuaValue>()
        .filter_map(|r| r.ok())
        .map(|(k, _)| k)
        .collect();

    for key in &current_keys {
        if !snapshot.theme_keys.contains(key) {
            theme.set(key.as_str(), LuaValue::Nil)?;
        }
    }

    Ok(())
}

struct PluginSnapshot {
    hook_count: usize,
    theme_keys: Vec<String>,
}

fn compute_load_order(specs: &[PluginSpec]) -> Vec<(&PluginSpec, bool)> {
    let mut order = Vec::new();
    let mut seen = HashSet::new();

    for spec in specs {
        for dep in &spec.dependencies {
            if seen.insert(dep.url.clone()) {
                order.push((dep, true));
            }
        }
        if seen.insert(spec.url.clone()) {
            order.push((spec, false));
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
}
