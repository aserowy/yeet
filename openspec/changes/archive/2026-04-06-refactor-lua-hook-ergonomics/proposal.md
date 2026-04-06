## Why

The Lua hook integration has several ergonomic issues: `yeet_lua::Lua` is used as a raw type alias throughout the codebase, hook calls use awkward `super::super::hook::` paths, and the hook invocation pattern in help/quickfix/tasks has `_ => return Vec::new()` early returns that could silently swallow function execution if a bug changes the Window variant. These issues make the code harder to read and risk subtle bugs.

## What Changes

- Rename the re-exported `yeet_lua::Lua` to `LuaConfiguration` for clarity at call sites
- Fix hook module visibility so all call sites use `crate::update::hook::on_window_create` instead of relative `super::super::hook::` paths
- Remove the defensive `_ => return Vec::new()` early returns in hook invocation blocks for help, quickfix, and tasks — the hook function takes `&mut Window` and never changes the variant, so these branches are unreachable. Replace with `unreachable!()` or restructure to avoid the pattern
- Add tests that exercise hook call sites with a non-None Lua instance to verify that function behavior (window creation, split structure, buffer registration) is unchanged when hooks are active

## Capabilities

### New Capabilities

### Modified Capabilities

## Impact

- **yeet-lua crate** (`lib.rs`): Add `pub type LuaConfiguration = mlua::Lua` type alias
- **yeet-frontend crate**: Replace all `yeet_lua::Lua` with `yeet_lua::LuaConfiguration`
- **yeet crate** (`lua.rs`, `main.rs`): Update type references
- **yeet-frontend/src/update/command/**: Fix `super::super::hook` paths to `crate::update::hook`, remove unreachable early returns
- **Tests**: Add tests for help, quickfix, tasks, and split with active Lua hooks
