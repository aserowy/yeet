## Why

`plugin_states` and `plugin_concurrency` are currently stored directly on `Model`, making it the only place where plugin runtime state and configuration live. `plugin_states` belongs alongside other runtime state in `State`, and `plugin_concurrency` is a setting that belongs in `Settings`. Additionally, the settings initialization path should be refactored so that Lua-configurable values flow through a single pattern, making it easy to add more Lua-configurable settings later.

## What Changes

- Move `plugin_states: Vec<PluginState>` from `Model` to `State`
- Move `plugin_concurrency: usize` from `Model` to `Settings`
- Refactor `yeet_frontend::run()` signature to no longer accept `plugin_states` and `plugin_concurrency` as separate parameters — they flow through `Settings` and are set during initialization
- Introduce a pattern in `Settings` for Lua-configurable fields (read from Lua after init, with defaults)
- Update all call sites that read `model.plugin_states` → `model.state.plugin_states` and `model.plugin_concurrency` → `model.settings.plugin_concurrency`

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

_None — this is a pure internal refactor with no spec-level behavior changes_

## Impact

- **yeet-frontend**: `Model`, `State`, `Settings` structs change; `run()` signature changes; command dispatch reads from new locations
- **yeet (binary)**: `main.rs` and `lua.rs` adapt to new `run()` signature; settings initialization moves plugin_concurrency into `Settings`
- No user-facing behavior changes
