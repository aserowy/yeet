## 1. Add buffer_type_for_lua method on Buffer enum

- [x] 1.1 Add `pub fn buffer_type_for_lua(&self) -> &'static str` method on the `Buffer` enum in `yeet-frontend/src/model/mod.rs` that maps each variant to its Lua string representation: Directory→"directory", Content→"content", Image→"image", Help→"help", QuickFix→"quickfix", Tasks→"tasks", PathReference→"content", Empty→"empty"

## 2. Update invoke_on_window_change signature and implementation

- [x] 2.1 Change `invoke_on_window_change` and `try_invoke_on_window_change` in `yeet-lua/src/hook.rs` to accept `buffer_types: [Option<&str>; 3]` instead of `preview_is_directory: bool`
- [x] 2.2 In `try_invoke_on_window_change`, remove the `ctx.set("preview_is_directory", ...)` line and instead set `buffer_type` on each viewport subtable (parent, current, preview) from the `buffer_types` array

## 3. Update invoke_on_window_change_for_focused helper

- [x] 3.1 In `yeet-frontend/src/update/hook.rs`, replace the `is_directory` resolution logic with buffer type resolution for all three viewports (parent, current, preview) using `buffer_type_for_lua()`
- [x] 3.2 Pass the resolved buffer types as `[Option<&str>; 3]` to `yeet_lua::invoke_on_window_change`

## 4. Update directory-icons plugin

- [x] 4.1 In `plugins/directory-icons/init.lua`, change the `on_window_change` callback from `ctx.preview_is_directory` to `ctx.preview.buffer_type == "directory"`

## 5. Update tests

- [x] 5.1 Update Rust tests in `yeet-lua/src/hook.rs` that reference `preview_is_directory` to use the new `buffer_types` parameter and verify `buffer_type` is set on subtables
- [x] 5.2 Update Rust tests in `yeet-frontend/src/update/hook.rs` that reference `preview_is_directory` to use the new signature

## 6. Update documentation

- [x] 6.1 Update `docs/help/hooks.md` to replace `preview_is_directory` references with `buffer_type` documentation on viewport subtables
- [x] 6.2 Run `markdownlint` on all changed markdown files in `./docs` and fix any issues

## 7. Verify

- [x] 7.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [x] 7.2 Run `cargo test` and fix any test failures
- [x] 7.3 Run `git add -A && nix build .` and fix any build failures
