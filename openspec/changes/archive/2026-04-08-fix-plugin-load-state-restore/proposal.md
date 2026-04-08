## Why

The plugin load snapshot/restore in `loading.rs` uses a length-based approach for hook rollback: it records the hook count before loading and removes entries beyond that count on failure. This is fragile — if a future hook API allows replacing or removing entries (not just appending), the restore would silently corrupt state. The theme restore is similarly ad-hoc, tracking key sets. Replacing both with a full shallow-clone of the hook and theme tables before each plugin load simplifies the logic and makes it correct regardless of how plugins modify state.

## What Changes

- Replace `PluginSnapshot { hook_count, theme_keys }` with cloned Lua tables (shallow copy of `y.hook.on_window_create` entries and `y.theme` entries)
- On failure, replace the live table contents with the cloned data instead of doing arithmetic on lengths/keys
- Add a test verifying that hooks registered by earlier plugins survive a later plugin's failure

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

_None — bugfix/hardening of existing rollback behavior_

## Impact

- **yeet-lua/src/loading.rs**: `take_snapshot` and `restore_snapshot` rewritten; `PluginSnapshot` struct changes
