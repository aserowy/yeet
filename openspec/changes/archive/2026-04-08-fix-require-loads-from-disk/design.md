## Context

The custom searcher currently checks if a module name matches a registered plugin and returns a no-op proxy. It has no knowledge of the filesystem — it doesn't know if the plugin is already downloaded.

Plugin data path is resolved in Rust via `yeet_plugin::resolve_plugin_data_path()`. The URL-to-storage-path mapping is `url_to_storage_path()` which extracts `owner/repo` from the URL.

## Goals / Non-Goals

**Goals:**

- Searcher loads plugin from disk when available (returns real module from `dofile()`)
- Searcher returns no-op proxy only when plugin is not on disk (fresh install)
- Theme values and hooks set via `require('plugin').setup()` in user's `init.lua` persist

**Non-Goals:**

- Snapshot/rollback for plugins loaded via the searcher (they're explicitly required by the user)
- Dependency ordering for searcher-loaded plugins

## Decisions

### 1. Set data path on y.plugin table from Rust

Before executing `init.lua`, set `y.plugin._data_path` from Rust using `resolve_plugin_data_path()`. If the path can't be resolved, leave it nil.

### 2. URL-to-storage-path in Lua

The searcher replicates the Rust `url_to_storage_path` logic in Lua:
```lua
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
```

### 3. Searcher loads from disk or returns proxy

```lua
local function plugin_searcher(modname)
    -- find matching registered plugin
    -- if data_path and init.lua exists on disk:
    --   dofile(init_path) → store in package.loaded → return real module
    -- else:
    --   return no-op proxy (fresh install)
end
```

`dofile()` executes the file and returns its return value. The searcher stores it in `package.loaded[modname]` so subsequent `require()` calls return the cached module.

### 4. Plugins loaded via searcher skip load_plugins

In `load_plugins`, if `package.loaded[plugin_name]` is already set (because the searcher loaded it during `init.lua`), skip re-loading that plugin. This prevents double execution.
