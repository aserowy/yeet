## Why

When a user's `init.lua` contains `y.plugin.register(...)` followed by `require('plugin-name').setup()`, the `require()` fails on a fresh install because the plugin isn't downloaded yet. The Lua error causes `init()` to return `None`, discarding the entire Lua runtime — including all `register()` calls made before the error.

## What Changes

- Register a custom Lua package searcher for registered plugin names that returns a no-op proxy table when the plugin isn't loaded yet
- The proxy uses `__index` metamethod to return no-op functions for any method call, so `require('bluloco-theme').setup()` silently does nothing on fresh install
- Once the plugin is loaded (after `:pluginsync` or `:pluginupdate` and restart), `require()` returns the real module table

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `plugins`: `require()` of a registered but not-yet-loaded plugin returns a no-op proxy instead of erroring

## Impact

- **yeet-lua/src/plugin.rs** or **yeet-lua/src/lib.rs**: Custom package searcher added during Lua setup that checks registered plugin names
