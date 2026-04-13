## 1. Register on_window_change hook in Lua bootstrap

- [ ] 1.1 In `yeet-lua/src/lib.rs`, add `on_window_change` hook object creation in `setup_and_execute` alongside `on_window_create` and `on_bufferline_mutate`, using the shared `hook_mt` metatable
- [ ] 1.2 Run `cargo test -p yeet-lua` to verify the new hook object is accessible

## 2. Implement invoke_on_window_change in yeet-lua

- [ ] 2.1 In `yeet-lua/src/hook.rs`, add `invoke_on_window_change` and `try_invoke_on_window_change` functions that mirror `invoke_on_window_create` but read from `y.hook.on_window_change`. The context builder SHALL include a `preview_is_directory` boolean field in addition to the standard directory context fields
- [ ] 2.2 Export `invoke_on_window_change` in `yeet-lua/src/lib.rs`
- [ ] 2.3 Add unit tests in `yeet-lua/src/hook.rs` for: callback invocation, viewport read-back, preview_is_directory field, error handling, no callbacks registered

## 3. Call on_window_change from preview::set_buffer_id

- [ ] 3.1 Extend `preview::set_buffer_id` signature to accept `Option<&LuaConfiguration>` and the `&mut Window` reference needed to pass all three viewports to the hook
- [ ] 3.2 Inside `preview::set_buffer_id`, after setting `hide_cursor_line`, invoke `yeet_lua::invoke_on_window_change` with the directory window's three viewports, the current path, and the `is_directory` boolean
- [ ] 3.3 Update all call sites of `preview::set_buffer_id` in `selection.rs` to pass the Lua runtime parameter
- [ ] 3.4 Thread the Lua runtime through to `selection::set_preview_buffer_for_selection` and `selection::refresh_preview_from_selection` and `selection::refresh_preview_from_current_selection` from all callers (cursor.rs, enumeration.rs, path.rs, modify.rs, viewport.rs, navigate.rs)

## 4. Update frontend hook.rs for on_window_change

- [ ] 4.1 Add `on_window_change` function in `yeet-frontend/src/update/hook.rs` that extracts viewports from a Directory Window and calls `yeet_lua::invoke_on_window_change`, or use the invocation directly in `preview::set_buffer_id`

## 5. Update directory-icons plugin

- [ ] 5.1 In `plugins/directory-icons/init.lua`, remove `ctx.preview.prefix_column_width = 2` from the `on_window_create` callback
- [ ] 5.2 Add an `on_window_change` callback that sets `ctx.preview.prefix_column_width = 2` when `ctx.preview_is_directory == true` and `ctx.preview.prefix_column_width = 0` when `ctx.preview_is_directory == false`

## 6. Update Rust tests for on_window_change hook

- [ ] 6.1 Update existing tests in `yeet-lua/src/hook.rs` that reference the hook list to include `on_window_change` where applicable
- [ ] 6.2 Add integration tests in `yeet-frontend/src/update/hook.rs` for `on_window_change` invocation from directory window context

## 7. Validation

- [ ] 7.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [ ] 7.2 Run `cargo test` and fix any failures
- [ ] 7.3 Run `git add -A && nix build .` and fix any build errors
