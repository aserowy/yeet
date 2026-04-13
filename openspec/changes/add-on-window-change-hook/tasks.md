## 1. Register on_window_change hook in Lua bootstrap

- [x] 1.1 In `yeet-lua/src/lib.rs`, add `on_window_change` hook object creation in `setup_and_execute` alongside `on_window_create` and `on_bufferline_mutate`, using the shared `hook_mt` metatable
- [x] 1.2 Run `cargo test -p yeet-lua` to verify the new hook object is accessible

## 2. Implement invoke_on_window_change in yeet-lua

- [x] 2.1 In `yeet-lua/src/hook.rs`, add `invoke_on_window_change` and `try_invoke_on_window_change` functions that mirror `invoke_on_window_create` but read from `y.hook.on_window_change`. The context builder SHALL include a `preview_is_directory` boolean field in addition to the standard directory context fields
- [x] 2.2 Export `invoke_on_window_change` in `yeet-lua/src/lib.rs`
- [x] 2.3 Add unit tests in `yeet-lua/src/hook.rs` for: callback invocation, viewport read-back, preview_is_directory field, error handling, no callbacks registered

## 3. Invoke on_window_change at end of update::model()

- [x] 3.1 In `yeet-frontend/src/update/mod.rs`, add `invoke_on_window_change` call at the end of `update::model()` after `buffers::update` and before `register::finish_scope`. The invocation SHALL get the current directory window's buffer IDs, determine `current_path` and `is_directory` from the buffer contents, then call `yeet_lua::invoke_on_window_change` with the three viewports
- [x] 3.2 Ensure `preview::set_buffer_id` does NOT invoke the hook (clean separation: preview.rs only sets buffer_id and hide_cursor_line)
- [x] 3.3 Remove `lua: Option<&LuaConfiguration>` parameter from `preview::set_buffer_id`, `selection::refresh_preview_from_current_selection`, `selection::refresh_preview_from_selection`, `selection::set_preview_buffer_for_selection`
- [x] 3.4 Remove `lua` parameter threading from `cursor::relocate`, `viewport::relocate`, `navigate::mark`, `navigate::path`, `navigate::path_as_preview`, `navigate::navigate_to_path_with_selection`, `navigate::selected`, and `path::remove`
- [x] 3.5 Remove `lua` parameter threading from internal `path.rs` functions: `update_directory_buffers_on_remove`, `cleanup_removed_buffers`, `cleanup_removed_buffers_in_window`, `update_viewports_for_buffers`, `update_viewports_for_buffers_in_window`
- [x] 3.6 Update all call sites in `mod.rs`, `mode.rs`, `enumeration.rs`, `modify.rs` to remove the `lua` argument from the above functions

## 4. Update frontend hook.rs tests for on_window_change

- [x] 4.1 Update test helper in `yeet-frontend/src/update/hook.rs` to include `on_window_change` hook object
- [x] 4.2 Add integration tests in `yeet-frontend/src/update/hook.rs` for `on_window_change` invocation from directory window context

## 5. Update directory-icons plugin

- [x] 5.1 In `plugins/directory-icons/init.lua`, remove `ctx.preview.prefix_column_width = 2` from the `on_window_create` callback
- [x] 5.2 Add an `on_window_change` callback that sets `ctx.preview.prefix_column_width = 2` when `ctx.preview_is_directory == true` and `ctx.preview.prefix_column_width = 0` when `ctx.preview_is_directory == false`

## 6. Validation

- [x] 6.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [x] 6.2 Run `cargo test` and fix any failures
- [x] 6.3 Run `git add -A && nix build .` and fix any build errors
