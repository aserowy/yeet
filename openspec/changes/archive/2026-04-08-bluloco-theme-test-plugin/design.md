## Context

The plugin loading system executes each plugin's `init.lua` via `lua.load(&content).exec()`. This discards any return value. Lua's standard module pattern is `require('module-name')` which returns a table — but `require()` only works if the module's directory is in `package.path` and the returned value is stored in `package.loaded`.

## Goals / Non-Goals

**Goals:**

- Plugin `init.lua` can return a module table with a `setup()` function
- Users can call `require('plugin-name').setup()` from their `init.lua`
- The bluloco-theme plugin demonstrates this pattern

**Non-Goals:**

- Complex module resolution (just `init.lua` at plugin root)
- Plugin dependency require chains (plugins requiring other plugins)

## Decisions

### 1. Plugin name derivation

The plugin name for `require()` is derived from the git URL: the last path segment with `yeet-` prefix stripped. For `https://github.com/aserowy/yeet-bluloco-theme`, the require name is `bluloco-theme`. This is the same as the repo directory name under `<owner>/`.

### 2. Add plugin directory to package.path

Before executing a plugin's `init.lua`, prepend its directory to `package.path`:
```
<plugin_dir>/?.lua;<plugin_dir>/?/init.lua;<existing_path>
```

This lets the plugin's own `init.lua` and any sibling `.lua` files be `require()`-able.

### 3. Use eval() and store in package.loaded

Change `lua.load(&content).exec()` to `lua.load(&content).eval::<LuaValue>()`. If the return value is a non-nil table, store it in `package.loaded[plugin_name]`. This makes `require('bluloco-theme')` return the module table without re-executing `init.lua`.

### 4. Plugin init.lua pattern

```lua
local M = {}

function M.setup()
    -- apply theme, register hooks, etc.
end

return M
```

Users in their `init.lua`:
```lua
y.plugin.register({ url = "https://github.com/aserowy/yeet-bluloco-theme" })
-- after plugins are loaded:
require('bluloco-theme').setup()
```

Note: since plugins are loaded after `init.lua` but before UI, the `require()` call needs to happen in a plugin's own `init.lua` or via a post-load hook. For the initial version, the plugin's `init.lua` can call `setup()` itself — making the `setup()` pattern optional but available.

### 5. Bluloco theme plugin structure

The plugin's `init.lua` defines the palette, creates the module with `setup()`, and calls `setup()` automatically so it works without user intervention. Users who want to customize can skip the auto-call and use `require('bluloco-theme').setup()` with options later.

For now: `init.lua` applies the theme directly (no setup() indirection needed for a theme-only plugin). The `setup()` function is exported for documentation purposes.
