## Context

The hook system currently has two hooks: `on_window_create` (fires once at window creation time) and `on_bufferline_mutate` (fires per-line during buffer population). Neither fires when window meta-information changes during navigation — specifically, when the preview target changes from a directory to a file or vice versa, or when navigation changes viewport assignments.

The directory-icons plugin currently sets `prefix_column_width = 2` on all three panes (parent, current, preview) unconditionally in `on_window_create`. This wastes space when the preview shows file content, since file previews have no icons in the prefix column.

The Lua hook bootstrap in `yeet-lua/src/lib.rs` creates hook objects with a shared metatable that provides `:add()`. Hook invocation in `yeet-lua/src/hook.rs` follows a pattern: build context table → iterate callbacks → read back viewport settings.

Public functions in the `update` module (`navigate::*`, `cursor::relocate`, `viewport::relocate`, `enumeration::change`, `enumeration::finish`, `path::add`, `path::remove`, `modify::buffer`) are the entry points that mutate viewport paths and buffer assignments. Each is called from `update::model()` or `mode.rs`.

## Goals / Non-Goals

**Goals:**
- Add `on_window_change` hook that fires at the end of each public function that actually changes viewport paths or buffer assignments
- Reuse the same context structure as `on_window_create` (type, path, parent/current/preview viewports)
- Ensure the hook fires for all viewport changes: navigation, cursor movement, preview changes, enumeration, path add/remove, buffer modifications
- Enable the directory-icons plugin to conditionally set `prefix_column_width = 2` on preview only when the preview target is a directory

**Non-Goals:**
- Firing `on_window_change` for non-Directory window types (Help, QuickFix, Tasks do not have dynamic meta-information changes)
- Adding new fields to the context table beyond what `on_window_create` already provides (except `preview_is_directory`)
- Adding a generic "what changed" descriptor to the context — plugins inspect current state, not diffs

## Decisions

### Decision 1: Hook fires at the end of each public function that changes viewport paths or buffer assignments

The `on_window_change` hook fires at the end of each public function that actually mutates viewport paths or buffer assignments. The affected functions are:

- `navigate::mark`, `navigate::path`, `navigate::path_as_preview`, `navigate::navigate_to_path_with_selection`, `navigate::parent`, `navigate::selected`
- `cursor::relocate` (Directory branch only)
- `viewport::relocate` (Directory branch only)
- `enumeration::change`, `enumeration::finish`
- `path::add`, `path::remove`
- `modify::buffer` (Directory branch only)

This per-function invocation ensures the hook fires immediately after each state change, giving plugins the most up-to-date viewport state at each invocation point.

**Alternative (iteration 2, rejected)**: Fire once at the end of `update::model()` as a central invocation. Rejected because it fires on every update cycle regardless of whether viewport paths actually changed, and it creates a single point of invocation that is disconnected from the actual state mutations.

**Alternative (iteration 1, rejected)**: Fire only from `preview::set_buffer_id`. Rejected because it misses navigation events like `navigate::parent` that swap viewports without going through preview refresh.

### Decision 2: Reuse `build_context` and `read_back_context` from on_window_create

The `on_window_change` invocation reuses the same `build_context` and `read_back_context` functions from the existing `on_window_create` implementation. The context table has the same structure: `{ type = "directory", path = "<current_path>", parent = {...}, current = {...}, preview = {...} }`.

**Alternative**: Create a separate context builder with "what changed" metadata. Rejected — plugins should inspect current state and decide based on that.

### Decision 3: Cycle prevention via per-function invocation placement

Cycle prevention is achieved by placement: the hook fires at the end of each function, after all mutations within that function are complete. The hook can modify viewport settings (like `prefix_column_width`), but these modifications do not trigger additional function calls or re-invoke the hook. There is no re-entrancy path because the hook runs after the function's mutations are finalized and before it returns.

### Decision 4: Lua runtime threaded through affected functions via Option<&LuaConfiguration>

`lua: Option<&LuaConfiguration>` is threaded through each affected function so the hook can be invoked directly at each call site. This adds the `lua` parameter to: `navigate::mark`, `navigate::path`, `navigate::path_as_preview`, `navigate::navigate_to_path_with_selection`, `navigate::parent`, `navigate::selected`, `cursor::relocate`, `viewport::relocate`, `path::remove`. Functions that already had the `lua` parameter (`enumeration::change`, `enumeration::finish`, `path::add`, `modify::buffer`) simply use it for the new hook invocation.

Callers in `mod.rs` and `mode.rs` pass `model.lua.as_ref()` at each call site.

**Alternative (iteration 2, rejected)**: Avoid threading `lua` and invoke the hook centrally at the end of `update::model()`. Rejected in favor of per-function invocation for more precise hook firing.

### Decision 5: Helper function `invoke_on_window_change_for_focused` centralizes invocation logic

A helper function `invoke_on_window_change_for_focused` in `yeet-frontend/src/update/hook.rs` encapsulates the repeated logic needed at each call site: get focused directory buffer IDs, resolve current path, determine `preview_is_directory`, get mutable viewports, and call `yeet_lua::invoke_on_window_change`. This avoids duplicating ~15 lines of boilerplate at every invocation point.

### Decision 6: Plugin changes preview prefix_column_width in on_window_change

The directory-icons plugin registers an `on_window_change` callback that checks whether the preview buffer is a directory (via `ctx.preview_is_directory`). The plugin sets `ctx.preview.prefix_column_width = 2` when the preview target is a directory, and `ctx.preview.prefix_column_width = 0` when it is not.

The `on_window_create` hook continues to set `prefix_column_width = 2` on parent and current (since those always show directories), but no longer sets it on preview.

### Decision 7: Context includes preview_is_directory flag

To enable plugins to determine whether the preview is showing a directory without filesystem access from Lua, the context table includes a `preview_is_directory` boolean field. This is derived from the preview buffer type in the application state. This avoids the need for Lua-side path inspection or filesystem access.

## Risks / Trade-offs

- [Performance] Hook fires at the end of each affected function, which means multiple invocations per update cycle if multiple functions are called (e.g., navigation triggers both `navigate::selected` and `cursor::relocate`). Mitigation: Lua callback execution is lightweight; the context table creation is cheap since it reuses existing functions.
- [Plugin compatibility] Existing `on_window_create` plugins that set preview `prefix_column_width` will still work — their setting just gets overridden by `on_window_change` on the next invocation. This is the desired behavior.
- [API surface] Adding a new hook increases the API surface. Mitigation: The hook follows the exact same pattern as `on_window_create`, minimizing cognitive overhead.
- [Parameter threading] Adding `lua: Option<&LuaConfiguration>` to multiple function signatures increases coupling. Mitigation: The parameter is optional (`Option`) and the helper function keeps invocation logic centralized.
