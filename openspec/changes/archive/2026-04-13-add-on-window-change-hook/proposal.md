## Why

The `on_window_create` hook fires only once when a window is created, but plugins need to react to ongoing changes like preview target changes (directory → file or vice versa) and navigation events that change viewport assignments. Currently, the directory-icons plugin sets `prefix_column_width = 2` on the preview viewport unconditionally at window creation, wasting space when the preview shows file content instead of a directory listing. A new `on_window_change` hook is needed so plugins can dynamically adjust viewport settings whenever window state changes.

## What Changes

- Add a new `on_window_change` Lua hook that fires at the end of each public function that actually changes viewport paths or buffer assignments (navigate, cursor, viewport, enumeration, path, modify)
- The hook receives the same context structure as `on_window_create` (type, path, parent/current/preview viewports) plus a `preview_is_directory` boolean
- A helper function `invoke_on_window_change_for_focused` in `hook.rs` centralizes the invocation logic (buffer ID lookup, path resolution, viewport retrieval) to avoid code duplication across call sites
- `lua: Option<&LuaConfiguration>` is threaded through the affected functions so each call site can invoke the hook directly before returning
- Update the directory-icons plugin to use `on_window_change` to conditionally set `prefix_column_width = 2` on the preview viewport only when the preview target is a directory

## Capabilities

### New Capabilities
- `on-window-change-hook`: A new Lua hook that fires at the end of each function that changes viewport paths or buffer assignments, enabling plugins to react dynamically to navigation and preview changes

### Modified Capabilities
- `lua`: Add `on_window_change` to the hook namespace alongside `on_window_create`, with the same context structure and read-back semantics
- `directory-icons-plugin`: Update the plugin to use `on_window_change` to conditionally set preview `prefix_column_width` based on whether the preview target is a directory

## Impact

- `yeet-lua/src/hook.rs`: New `invoke_on_window_change` and `try_invoke_on_window_change` functions with `preview_is_directory` context field
- `yeet-lua/src/lib.rs`: New public export for `invoke_on_window_change`, new hook object registration in `setup_and_execute`
- `yeet-frontend/src/update/hook.rs`: New `invoke_on_window_change_for_focused` helper function; integration tests for `on_window_change` invocation
- `yeet-frontend/src/update/navigate.rs`: `lua` parameter added to `mark`, `path`, `path_as_preview`, `navigate_to_path_with_selection`, `parent`, `selected`; hook invoked before return
- `yeet-frontend/src/update/cursor.rs`: `lua` parameter added to `relocate`; hook invoked in Directory branch
- `yeet-frontend/src/update/viewport.rs`: `lua` parameter added to `relocate`; hook invoked in Directory branch
- `yeet-frontend/src/update/enumeration.rs`: Hook invoked before return in `change` and `finish` (already had `lua`)
- `yeet-frontend/src/update/path.rs`: `lua` parameter added to `remove`; hook invoked before return in `add` and `remove`
- `yeet-frontend/src/update/modify.rs`: Hook invoked in `buffer` Directory branch (already had `lua`)
- `yeet-frontend/src/update/mod.rs`: Updated callers to pass `lua` to navigate, cursor, viewport, path functions
- `yeet-frontend/src/update/mode.rs`: Updated `flush_pending_paths` to pass `lua` to `path::remove`
- `plugins/directory-icons/init.lua`: Move preview `prefix_column_width` logic from `on_window_create` to `on_window_change`
- Lua bootstrap code: Register `on_window_change` as a hook object on `y.hook`
