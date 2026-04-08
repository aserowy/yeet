use mlua::prelude::*;
use yeet_plugin::PluginSpec;

pub fn create_plugin_table(lua: &Lua) -> LuaResult<LuaTable> {
    let plugin_table = lua.create_table()?;

    let plugins_list = lua.create_table()?;
    plugin_table.set("_plugins", plugins_list)?;

    plugin_table.set("concurrency", 4)?;

    let register_fn = lua.create_function(|lua, opts: LuaValue| {
        let table = match opts {
            LuaValue::Table(t) => t,
            _ => {
                tracing::error!(
                    "y.plugin.register() called with {}, expected table",
                    opts.type_name()
                );
                return Ok(());
            }
        };

        let url: Option<String> = table.get("url").ok();
        let url = match url {
            Some(u) if !u.is_empty() => u,
            _ => {
                tracing::error!("y.plugin.register() called without 'url' field");
                return Ok(());
            }
        };

        let name: Option<String> = table.get("name").ok();
        let branch: Option<String> = table.get("branch").ok();
        let version: Option<String> = table.get("version").ok();

        let deps_table: Option<LuaTable> = table.get("dependencies").ok();
        let dependencies = if let Some(deps) = deps_table {
            parse_dependencies(lua, &deps)?
        } else {
            Vec::new()
        };

        let entry = lua.create_table()?;
        entry.set("url", url)?;
        if let Some(ref n) = name {
            entry.set("name", n.clone())?;
        }
        if let Some(ref b) = branch {
            entry.set("branch", b.clone())?;
        }
        if let Some(ref v) = version {
            entry.set("version", v.clone())?;
        }

        let deps_lua = lua.create_table()?;
        for (i, dep) in dependencies.iter().enumerate() {
            let dep_entry = lua.create_table()?;
            dep_entry.set("url", dep.url.clone())?;
            if let Some(ref b) = dep.branch {
                dep_entry.set("branch", b.clone())?;
            }
            if let Some(ref v) = dep.version {
                dep_entry.set("version", v.clone())?;
            }
            deps_lua.set(i + 1, dep_entry)?;
        }
        entry.set("dependencies", deps_lua)?;

        let globals = lua.globals();
        let y: LuaTable = globals.get("y")?;
        let plugin: LuaTable = y.get("plugin")?;
        let plugins: LuaTable = plugin.get("_plugins")?;
        let len = plugins.raw_len();
        plugins.raw_set(len + 1, entry)?;

        Ok(())
    })?;

    plugin_table.set("register", register_fn)?;

    Ok(plugin_table)
}

fn parse_dependencies(_lua: &Lua, deps: &LuaTable) -> LuaResult<Vec<PluginSpec>> {
    let mut result = Vec::new();

    for pair in deps.pairs::<i64, LuaTable>() {
        let (_, dep_table) = pair?;

        let url: Option<String> = dep_table.get("url").ok();
        let url = match url {
            Some(u) if !u.is_empty() => u,
            _ => {
                tracing::warn!("dependency without 'url' field, skipping");
                continue;
            }
        };

        let nested_deps: Option<LuaTable> = dep_table.get("dependencies").ok();
        if nested_deps.is_some() {
            tracing::warn!(
                "dependency '{}' has nested dependencies, ignoring them",
                url
            );
        }

        result.push(PluginSpec {
            url,
            name: dep_table.get("name").ok(),
            branch: dep_table.get("branch").ok(),
            version: dep_table.get("version").ok(),
            dependencies: Vec::new(),
        });
    }

    Ok(result)
}

pub fn read_plugin_specs(lua: &Lua) -> Vec<PluginSpec> {
    let Ok(y) = lua.globals().get::<LuaTable>("y") else {
        return Vec::new();
    };
    let Ok(plugin) = y.get::<LuaTable>("plugin") else {
        return Vec::new();
    };
    let Ok(plugins) = plugin.get::<LuaTable>("_plugins") else {
        return Vec::new();
    };

    let mut specs = Vec::new();
    for pair in plugins.pairs::<i64, LuaTable>().flatten() {
        let (_, entry) = pair;

        let url: String = match entry.get("url") {
            Ok(u) => u,
            Err(_) => continue,
        };

        let name: Option<String> = entry.get("name").ok();
        let branch: Option<String> = entry.get("branch").ok();
        let version: Option<String> = entry.get("version").ok();

        let dependencies = match entry.get::<LuaTable>("dependencies") {
            Ok(deps) => read_deps_from_table(&deps),
            Err(_) => Vec::new(),
        };

        specs.push(PluginSpec {
            url,
            name,
            branch,
            version,
            dependencies,
        });
    }

    specs
}

fn read_deps_from_table(deps: &LuaTable) -> Vec<PluginSpec> {
    let mut result = Vec::new();
    for pair in deps.pairs::<i64, LuaTable>().flatten() {
        let (_, dep) = pair;
        let url: String = match dep.get("url") {
            Ok(u) => u,
            Err(_) => continue,
        };
        result.push(PluginSpec {
            url,
            name: dep.get("name").ok(),
            branch: dep.get("branch").ok(),
            version: dep.get("version").ok(),
            dependencies: Vec::new(),
        });
    }
    result
}

pub fn read_plugin_concurrency(lua: &Lua) -> usize {
    let Ok(y) = lua.globals().get::<LuaTable>("y") else {
        return 4;
    };
    let Ok(plugin) = y.get::<LuaTable>("plugin") else {
        return 4;
    };

    match plugin.get::<LuaValue>("concurrency") {
        Ok(LuaValue::Integer(n)) if n > 0 => n as usize,
        Ok(LuaValue::Integer(_)) => {
            tracing::warn!("y.plugin.concurrency must be positive, using default 4");
            4
        }
        Ok(LuaValue::Nil) => 4,
        Ok(other) => {
            tracing::warn!(
                "y.plugin.concurrency expected integer, got {}, using default 4",
                other.type_name()
            );
            4
        }
        Err(_) => 4,
    }
}
