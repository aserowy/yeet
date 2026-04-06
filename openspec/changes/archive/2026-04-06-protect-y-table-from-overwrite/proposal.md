## Why

Users naturally write `y = { theme = { ... } }` in their `init.lua`, which overwrites the entire `y` table that Rust pre-created — including `y.hook` with its metatable. This causes `y.hook.on_window_create:add(...)` to error with "attempt to index a nil value (field 'hook')", silently disabling all hooks. The Lua runtime returns `None` and the application starts with defaults.

## What Changes

- Protect the global `y` variable by setting a metatable on `_G` that intercepts `y = { ... }` assignments and deep-merges the new table into the existing `y` table instead of replacing it
- This makes `y = { theme = { TabBarActiveBg = "#ff0000" } }` equivalent to `y.theme.TabBarActiveBg = "#ff0000"` — the `y.hook` subtable and its metatable survive the assignment
- Update documentation to explain the merge behavior

## Capabilities

### New Capabilities

### Modified Capabilities

- `lua`: The `y` table is now protected from overwrite via a metatable on `_G`. Assignments to `y` merge into the existing table.

## Impact

- **yeet-lua crate** (`lib.rs`): `setup_and_execute` sets a metatable on `_G` with a `__newindex` metamethod that intercepts writes to `"y"` and merges instead of replacing
- **docs/help/configuration.md**: Document that `y = { ... }` merges into the existing table
