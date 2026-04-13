## Context

The hook system currently has two hooks: `on_window_create` (fires once at window creation time) and `on_bufferline_mutate` (fires per-line during buffer population). Neither fires when window meta-information changes during navigation — specifically, when the preview target changes from a directory to a file or vice versa.

The directory-icons plugin currently sets `prefix_column_width = 2` on all three panes (parent, current, preview) unconditionally in `on_window_create`. This wastes space when the preview shows file content, since file previews have no icons in the prefix column.

The Lua hook bootstrap in `yeet-lua/src/lib.rs` creates hook objects with a shared metatable that provides `:add()`. Hook invocation in `yeet-lua/src/hook.rs` follows a pattern: build context table → iterate callbacks → read back viewport settings.

Preview buffer changes flow through `selection.rs::set_preview_buffer_for_selection` → `preview::set_buffer_id`, which already conditionally sets `hide_cursor_line` based on whether the preview target is a directory.

## Goals / Non-Goals

**Goals:**
- Add `on_window_change` hook that fires when window meta-information changes (preview target changes during navigation)
- Reuse the same context structure as `on_window_create` (type, path, parent/current/preview viewports)
- Implement cycle prevention so hook-triggered viewport changes do not cause re-invocation
- Enable the directory-icons plugin to conditionally set `prefix_column_width = 2` on preview only when the preview target is a directory

**Non-Goals:**
- Firing `on_window_change` for non-Directory window types (Help, QuickFix, Tasks do not have dynamic meta-information changes)
- Adding new fields to the context table beyond what `on_window_create` already provides
- Adding a generic "what changed" descriptor to the context — plugins inspect current state, not diffs

## Decisions

### Decision 1: Hook fires only for Directory windows after preview buffer assignment

The `on_window_change` hook fires after `preview::set_buffer_id` completes, which is the point where the preview target changes. This is the sole trigger point. The hook does not fire for Help/QuickFix/Tasks windows since their content does not change dynamically.

**Alternative**: Fire on any viewport field change. Rejected because it creates complexity with no current use case and risks performance overhead.

**Alternative**: Fire from cursor movement handlers. Rejected because `selection.rs` already centralizes preview refresh — hooking there covers all navigation paths (cursor movement, enumeration changes, path changes, navigation).

### Decision 2: Reuse `build_context` and `read_back_context` from on_window_create

The `on_window_change` invocation reuses the same `build_context` and `read_back_context` functions from the existing `on_window_create` implementation. The context table has the same structure: `{ type = "directory", path = "<current_path>", parent = {...}, current = {...}, preview = {...} }`.

**Alternative**: Create a separate context builder with "what changed" metadata. Rejected — plugins should inspect current state and decide based on that.

### Decision 3: Cycle prevention via single-shot invocation in set_buffer_id

Cycle prevention is achieved architecturally: the hook is called exactly once inside `preview::set_buffer_id`, after the buffer assignment is complete. The hook can modify viewport settings (like `prefix_column_width`), but viewport setting changes do not trigger `set_buffer_id` again. There is no re-entrancy path because:
1. `set_buffer_id` is called from `selection.rs` after buffer resolution
2. The hook fires, callbacks execute, viewport settings are read back
3. No code path re-invokes `set_buffer_id` based on viewport setting changes

This is inherent cycle prevention — no additional guard flag is needed.

### Decision 4: Hook invocation needs access to Lua runtime and Window

`preview::set_buffer_id` currently takes `(contents, window, buffer_id)`. To invoke the hook, it also needs access to the Lua runtime. The function signature is extended to accept `Option<&LuaConfiguration>`. When `None` (no Lua runtime), the hook is skipped.

The path for the context table is derived from the current viewport's buffer — specifically the path of the directory buffer that the current viewport is displaying.

### Decision 5: Plugin changes preview prefix_column_width in on_window_change

The directory-icons plugin registers an `on_window_change` callback that checks whether the preview buffer is a directory (by inspecting the preview path — if it ends with `/` or by checking if the path refers to a directory). The plugin sets `ctx.preview.prefix_column_width = 2` when the preview target is a directory, and `ctx.preview.prefix_column_width = 0` when it is not.

The `on_window_create` hook continues to set `prefix_column_width = 2` on parent and current (since those always show directories), but no longer sets it on preview.

### Decision 6: Context includes preview_is_directory flag

To enable plugins to determine whether the preview is showing a directory without filesystem access from Lua, the context table includes a `preview_is_directory` boolean field. This is derived from the same `is_directory` check already performed in `preview::set_buffer_id`. This avoids the need for Lua-side path inspection or filesystem access.

## Risks / Trade-offs

- [Performance] Hook fires on every preview change (every cursor movement in a directory). Mitigation: Lua callback execution is lightweight; the context table creation is cheap since it reuses existing functions.
- [Plugin compatibility] Existing `on_window_create` plugins that set preview `prefix_column_width` will still work — their setting just gets overridden by `on_window_change` on the next navigation. This is the desired behavior.
- [API surface] Adding a new hook increases the API surface. Mitigation: The hook follows the exact same pattern as `on_window_create`, minimizing cognitive overhead.
