## 1. Register on_window_change hook in Lua bootstrap

- [x] 1.1 In `yeet-lua/src/lib.rs`, add `on_window_change` hook object creation in `setup_and_execute` alongside `on_window_create` and `on_bufferline_mutate`, using the shared `hook_mt` metatable
- [x] 1.2 Run `cargo test -p yeet-lua` to verify the new hook object is accessible

## 2. Implement invoke_on_window_change in yeet-lua

- [x] 2.1 In `yeet-lua/src/hook.rs`, add `invoke_on_window_change` and `try_invoke_on_window_change` functions that mirror `invoke_on_window_create` but read from `y.hook.on_window_change`. The context builder SHALL include a `preview_is_directory` boolean field in addition to the standard directory context fields
- [x] 2.2 Export `invoke_on_window_change` in `yeet-lua/src/lib.rs`
- [x] 2.3 Add unit tests in `yeet-lua/src/hook.rs` for: callback invocation, viewport read-back, preview_is_directory field, error handling, no callbacks registered

## 3. Add invoke_on_window_change_for_focused helper in hook.rs

- [x] 3.1 In `yeet-frontend/src/update/hook.rs`, add `invoke_on_window_change_for_focused` helper function that encapsulates: get focused directory buffer IDs, resolve current path, determine `preview_is_directory`, get mutable viewports, and call `yeet_lua::invoke_on_window_change`
- [x] 3.2 Add integration tests in `yeet-frontend/src/update/hook.rs` for `on_window_change` invocation from directory window context

## 4. Invoke hook at end of each public function that changes viewport paths/buffers

- [x] 4.1 In `navigate.rs`, add `lua: Option<&LuaConfiguration>` parameter to `mark`, `path`, `path_as_preview`, `navigate_to_path_with_selection`, `parent`, `selected` and invoke `invoke_on_window_change_for_focused` before return
- [x] 4.2 In `cursor.rs`, add `lua: Option<&LuaConfiguration>` parameter to `relocate` and invoke hook in Directory branch
- [x] 4.3 In `viewport.rs`, add `lua: Option<&LuaConfiguration>` parameter to `relocate` and invoke hook in Directory branch
- [x] 4.4 In `enumeration.rs`, invoke hook before return in `change` and `finish` (already had `lua` parameter)
- [x] 4.5 In `path.rs`, add `lua: Option<&LuaConfiguration>` parameter to `remove` and invoke hook before return in `add` and `remove`
- [x] 4.6 In `modify.rs`, invoke hook in `buffer` Directory branch (already had `lua` parameter)

## 5. Update callers to pass lua parameter

- [x] 5.1 In `mod.rs`, update all call sites for navigate, cursor, viewport, and path functions to pass `model.lua.as_ref()`
- [x] 5.2 In `mode.rs`, update `flush_pending_paths` to pass `lua` to `path::remove`

## 6. Update directory-icons plugin

- [x] 6.1 In `plugins/directory-icons/init.lua`, remove `ctx.preview.prefix_column_width = 2` from the `on_window_create` callback
- [x] 6.2 Add an `on_window_change` callback that sets `ctx.preview.prefix_column_width = 2` when `ctx.preview_is_directory == true` and `ctx.preview.prefix_column_width = 0` when `ctx.preview_is_directory == false`

## 7. Validation

- [x] 7.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [x] 7.2 Run `cargo test` and fix any failures
- [x] 7.3 Run `git add -A && nix build .` and fix any build errors
