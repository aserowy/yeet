## Why

The custom package searcher always returns a no-op proxy for registered plugins because at `init.lua` execution time, `load_plugins` hasn't run yet. Even when the plugin is on disk (after a sync/update), `require('bluloco-theme').setup()` in the user's `init.lua` calls the no-op proxy's setup — theme values and hooks are silently discarded.

The plugin's own `init.lua` calls `setup()` during `load_plugins`, but the user has no way to control plugin configuration from their own `init.lua` via `require()`.

## What Changes

- Pass the plugin data path to the Lua searcher so it can find plugin directories
- The searcher checks if the plugin's `init.lua` exists on disk — if yes, loads it via `dofile()` and returns the real module; if not (fresh install), returns the no-op proxy
- This means `require('bluloco-theme').setup()` in the user's `init.lua` works on non-fresh installs

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `plugins`: `require()` of registered plugins loads from disk when available, falls back to no-op proxy when not downloaded

## Impact

- **yeet-lua/src/lib.rs**: Custom searcher gains data path awareness and `dofile()` loading
- **yeet-lua/src/plugin.rs** or **yeet-lua/src/lib.rs**: Data path passed to Lua as `y.plugin._data_path`
