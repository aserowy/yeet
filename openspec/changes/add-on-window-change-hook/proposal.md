## Why

The `on_window_create` hook fires only once when a window is created, but plugins need to react to ongoing changes like preview target changes (directory → file or vice versa) and navigation events that change viewport assignments. Currently, the directory-icons plugin sets `prefix_column_width = 2` on the preview viewport unconditionally at window creation, wasting space when the preview shows file content instead of a directory listing. A new `on_window_change` hook is needed so plugins can dynamically adjust viewport settings whenever window state changes.

## What Changes

- Add a new `on_window_change` Lua hook that fires once per update cycle at the end of `update::model()`, after all viewport mutations are complete
- The hook receives the same context structure as `on_window_create` (type, path, parent/current/preview viewports) plus a `preview_is_directory` boolean
- Cycle prevention is achieved architecturally: the hook fires at the end of the update cycle after all message processing is complete, so viewport modifications by callbacks do not trigger re-invocation
- Update the directory-icons plugin to use `on_window_change` to conditionally set `prefix_column_width = 2` on the preview viewport only when the preview target is a directory

## Capabilities

### New Capabilities
- `on-window-change-hook`: A new Lua hook that fires once per update cycle after all viewport mutations, enabling plugins to react dynamically to navigation and preview changes

### Modified Capabilities
- `lua`: Add `on_window_change` to the hook namespace alongside `on_window_create`, with the same context structure and read-back semantics
- `directory-icons-plugin`: Update the plugin to use `on_window_change` to conditionally set preview `prefix_column_width` based on whether the preview target is a directory

## Impact

- `yeet-lua/src/hook.rs`: New `invoke_on_window_change` and `try_invoke_on_window_change` functions with `preview_is_directory` context field
- `yeet-lua/src/lib.rs`: New public export for `invoke_on_window_change`, new hook object registration in `setup_and_execute`
- `yeet-frontend/src/update/mod.rs`: Invoke `on_window_change` at the end of `update::model()` after all message processing, window layout, and buffer updates are complete
- `yeet-frontend/src/update/hook.rs`: Integration tests for `on_window_change` invocation
- `plugins/directory-icons/init.lua`: Move preview `prefix_column_width` logic from `on_window_create` to `on_window_change`
- Lua bootstrap code: Register `on_window_change` as a hook object on `y.hook`
