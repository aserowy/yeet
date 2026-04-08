## 1. Move plugin_concurrency to Settings

- [x] 1.1 Add `plugin_concurrency: usize` field to `Settings` struct with default value 4
- [x] 1.2 Set `plugin_concurrency` from Lua in `yeet/src/lua.rs` on the `Settings` struct before passing to `run()`
- [x] 1.3 Remove `plugin_concurrency` field from `Model`

## 2. Move plugin_states to State

- [x] 2.1 Add `plugin_states: Vec<PluginState>` field to `State` struct (default empty Vec)
- [x] 2.2 Set `state.plugin_states` during Model initialization in `run()`
- [x] 2.3 Remove `plugin_states` field from `Model`

## 3. Simplify run() signature

- [x] 3.1 Change `run(settings, lua, plugin_states, plugin_concurrency)` to `run(settings, lua, plugin_states)`
- [x] 3.2 Update `main.rs` call site to pass 3 arguments (concurrency already in settings)

## 4. Refactor command dispatch to use Settings and State

- [x] 4.1 Change `command::execute` parameter `theme: &Theme` to `settings: &Settings` and drop `plugin_states` and `plugin_concurrency` params
- [x] 4.2 Update `execute` body to use `settings.theme` where `theme` was used, `state.plugin_states` where `plugin_states` was used, `settings.plugin_concurrency` where `plugin_concurrency` was used
- [x] 4.3 Update `update_with_keymap_message` caller to pass `settings` instead of `&settings.theme`
- [x] 4.4 Fix all test calls to `execute()` to match new signature

## 5. Build Verification

- [x] 5.1 Run `cargo fmt` and `cargo clippy` and fix all warnings
- [x] 5.2 Run `cargo test` and ensure all tests pass
- [x] 5.3 Run `nix build .` and ensure build succeeds
