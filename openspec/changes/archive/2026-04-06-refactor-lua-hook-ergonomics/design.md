## Context

`yeet_lua::Lua` is a re-export of `mlua::Lua`. It appears in 14 function signatures across `yeet-frontend` and `yeet` crates as `Option<yeet_lua::Lua>` or `Option<&yeet_lua::Lua>`. The name `Lua` doesn't convey that it's a configuration runtime.

Hook calls from `command/help.rs` and `command/split.rs` use `super::super::hook::on_window_create` because they're two levels deep from `update/`. Meanwhile `command/qfix/window.rs` already uses `crate::update::hook::on_window_create`, and `tab.rs`/`open.rs` use `super::hook::on_window_create`. There's no consistency.

The hook invocation pattern in help, quickfix, and tasks wraps the viewport in a temporary Window, calls the hook, then pattern-matches to extract the viewport back. The `_ =>` fallthrough branch does `return Vec::new()` — this is dead code since `on_window_create` takes `&mut Window` and never changes the enum variant. But if it ever ran, it would silently skip the entire window creation (no split, no buffer assignment).

All existing tests pass `None` for the lua parameter, so no test exercises the hook-active path in these functions.

## Goals / Non-Goals

**Goals:**

- Replace `yeet_lua::Lua` with `yeet_lua::LuaConfiguration` everywhere
- Standardize all hook call paths to `crate::update::hook::on_window_create`
- Remove the unreachable `_ => return Vec::new()` branches
- Add tests that pass a real Lua instance (with hooks) to verify window creation works correctly with active hooks

**Non-Goals:**

- Changing hook behavior or API
- Restructuring the hook module itself
- Moving hook invocation to different call sites

## Decisions

### 1. Type alias: `LuaConfiguration`

**Decision:** Add `pub type LuaConfiguration = mlua::Lua;` in `yeet-lua/src/lib.rs`. Replace `pub use mlua::Lua;` with `pub use mlua::Lua; pub type LuaConfiguration = Lua;`. Update all call sites to use `LuaConfiguration`.

### 2. Hook paths: use `crate::update::hook`

**Decision:** Each file that calls a hook adds `use crate::update::hook;` to its imports, then calls `hook::on_window_create(...)`. This gives clean `hook::` prefixed calls everywhere instead of varying `super::` chains. The module is already `pub`.

### 3. Remove unreachable early returns

**Decision:** The `on_window_create` function takes `&mut Window` — it reads viewport fields and writes them back but never changes the Window variant. The `_ => return Vec::new()` branches in help.rs, task.rs, and qfix/window.rs are unreachable. Remove them by restructuring: pass `&mut ViewPort` directly to the hook invocation instead of wrapping/unwrapping a temporary Window.

Actually, looking more carefully: `update/hook.rs` matches on the Window variant to determine context structure. The callers construct a temporary Window just so the hook module can pattern-match it. A cleaner approach: call `yeet_lua::invoke_on_window_create` directly with the window type string and viewport, bypassing the `update::hook::on_window_create` wrapper for single-viewport windows. But that leaks `yeet_lua` details into command modules.

**Chosen approach:** Keep the wrapper but change the temporary Window construction to not use `return Vec::new()`. Instead, since the variant cannot change, use `let Window::Help(vp) = help_window else { unreachable!() }` pattern (or equivalent). This documents the invariant without silently failing.

### 4. Tests with active Lua

**Decision:** For each hook call site (help, quickfix, tasks, split, tab, open), add a test that creates a real Lua runtime with a registered `on_window_create` hook, then verifies:
1. The function completes normally (no early return)
2. The hook's viewport modifications are applied
3. The window structure is correct (split created, buffer registered, etc.)

## Risks / Trade-offs

**`unreachable!()` panics in production** → Only if a bug makes the hook change the Window variant, which is impossible given the current `&mut Window` API. The panic is strictly better than a silent early return that skips window creation.
