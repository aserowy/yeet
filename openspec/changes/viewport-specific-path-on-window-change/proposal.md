## Why

The `on_window_change` hook currently exposes a single `path` property on the context table, set to the current directory's path. Plugins cannot determine the parent directory path or the preview target path without filesystem inspection from Lua. Adding per-viewport `path` properties enables plugins to react to each viewport's path independently.

## What Changes

- Change the `on_window_change` context table to include a `path` property on each viewport subtable (`ctx.parent.path`, `ctx.current.path`, `ctx.preview.path`) instead of a single top-level `ctx.path`
- Remove the top-level `ctx.path` from `on_window_change` (it remains on `on_window_create` unchanged)
- Resolve each viewport's path from its buffer: parent gets the parent directory path, current gets the current directory path, preview gets the preview target path
- Update `build_context` or the `on_window_change` invocation path to set per-viewport paths
- Update the `on_window_change` helper and tests

## Capabilities

### New Capabilities

### Modified Capabilities
- `lua`: Modify the `on_window_change` context table structure to include per-viewport `path` properties and remove top-level `ctx.path`

## Impact

- `yeet-lua/src/hook.rs`: Change `try_invoke_on_window_change` to accept per-viewport paths and set `path` on each viewport subtable instead of on the top-level context
- `yeet-lua/src/lib.rs`: Update `invoke_on_window_change` signature to accept per-viewport paths
- `yeet-frontend/src/update/hook.rs`: Update `invoke_on_window_change_for_focused` to resolve parent, current, and preview paths individually from their buffer IDs
- `yeet-frontend/src/update/hook.rs` tests: Update test assertions for per-viewport paths
- `yeet-lua/src/hook.rs` tests: Update unit tests for per-viewport path context
