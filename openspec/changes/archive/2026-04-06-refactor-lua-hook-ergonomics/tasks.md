## 1. Rename Lua to LuaConfiguration

- [x] 1.1 Add `pub type LuaConfiguration = Lua;` in `yeet-lua/src/lib.rs` (keep the `pub use mlua::Lua` for internal use)
- [x] 1.2 Update `invoke_on_window_create` signature in `yeet-lua/src/hook.rs` to use `LuaConfiguration`
- [x] 1.3 Update `read_theme_pairs` signature in `yeet-lua/src/lib.rs` to use `LuaConfiguration`
- [x] 1.4 Add `use yeet_lua::LuaConfiguration;` imports in all `yeet-frontend` files that reference the type, replacing inline `yeet_lua::LuaConfiguration`
- [x] 1.5 Add `use yeet_lua::LuaConfiguration;` imports in `yeet` crate (`lua.rs`), replacing inline `yeet_lua::LuaConfiguration`
- [x] 1.6 Update `update/hook.rs` to use `LuaConfiguration` instead of `Lua`

## 2. Standardize hook call paths

- [x] 2.1 In `command/help.rs`: add `use crate::update::hook;` and replace `super::super::hook::on_window_create` with `hook::on_window_create`
- [x] 2.2 In `command/split.rs`: add `use crate::update::hook;` and replace `super::super::hook::on_window_create` with `hook::on_window_create`
- [x] 2.3 In `update/tab.rs`: add `use crate::update::hook;` and replace `super::hook::on_window_create` with `hook::on_window_create`
- [x] 2.4 In `update/open.rs`: add `use crate::update::hook;` and replace `super::hook::on_window_create` with `hook::on_window_create`
- [x] 2.5 In `command/qfix/window.rs`: add `use crate::update::hook;` and replace `crate::update::hook::on_window_create` with `hook::on_window_create`

## 3. Remove unreachable early returns

- [x] 3.1 In `command/help.rs`: replace `_ => return Vec::new()` with an unreachable pattern after hook invocation
- [x] 3.2 In `command/task.rs`: replace `_ => return Vec::new()` with an unreachable pattern after hook invocation
- [x] 3.3 In `command/qfix/window.rs`: replace `_ => return Vec::new()` with an unreachable pattern after hook invocation

## 4. Tests: hook preserves Window variant

- [x] 4.1 Add tests in `update/hook.rs` that call `on_window_create` for each Window variant (Directory, Help, QuickFix, Tasks) with an active Lua hook and assert the Window variant is unchanged after invocation

## 5. Use imports instead of inline yeet_lua::LuaConfiguration

- [x] 5.1 In each file that uses `yeet_lua::LuaConfiguration` inline in signatures, add `use yeet_lua::LuaConfiguration;` and replace inline references with `LuaConfiguration`

## 6. Validation

- [x] 6.1 Run `cargo test` across the full workspace
- [x] 6.2 Run `cargo clippy` and `cargo fmt` and fix any issues
