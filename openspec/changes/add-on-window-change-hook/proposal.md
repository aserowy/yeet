## Why

The `on_window_create` hook fires only once when a window is created, but plugins need to react to ongoing changes like preview target changes (directory → file or vice versa). Currently, the directory-icons plugin sets `prefix_column_width = 2` on the preview viewport unconditionally at window creation, wasting space when the preview shows file content instead of a directory listing. A new `on_window_change` hook is needed so plugins can dynamically adjust viewport settings when window meta-information changes.

## What Changes

- Add a new `on_window_change` Lua hook that fires when window meta-information changes (e.g., preview target path changes, viewport buffer assignment changes)
- The hook receives the same context structure as `on_window_create` (type, path, parent/current/preview viewports) plus information about what changed
- Implement cycle prevention: after hook callbacks execute and viewport settings are read back, the hook does NOT re-fire even if the settings changed — only external state changes (path, buffer assignment) trigger the hook
- Update the directory-icons plugin to use `on_window_change` to conditionally set `prefix_column_width = 2` on the preview viewport only when the preview target is a directory

## Capabilities

### New Capabilities
- `on-window-change-hook`: A new Lua hook that fires when window meta-information changes, enabling plugins to react dynamically to navigation and preview changes

### Modified Capabilities
- `lua`: Add `on_window_change` to the hook namespace alongside `on_window_create`, with the same context structure and read-back semantics
- `directory-icons-plugin`: Update the plugin to use `on_window_change` to conditionally set preview `prefix_column_width` based on whether the preview target is a directory

## Impact

- `yeet-lua/src/hook.rs`: New `invoke_on_window_change` function mirroring `invoke_on_window_create`
- `yeet-lua/src/lib.rs`: New public export for `invoke_on_window_change`
- `yeet-frontend/src/update/preview.rs`: Call `invoke_on_window_change` when preview buffer changes in `set_buffer_id`
- `yeet-frontend/src/update/cursor.rs`: Call `invoke_on_window_change` when navigation changes the current/preview targets
- `yeet-frontend/src/update/hook.rs`: Registration of `on_window_change` hook alongside `on_window_create`
- `plugins/directory-icons/init.lua`: Move preview `prefix_column_width` logic from `on_window_create` to `on_window_change`
- Lua bootstrap code: Register `on_window_change` as a hook object on `y.hook`
