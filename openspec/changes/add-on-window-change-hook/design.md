## Context

The hook system currently has two hooks: `on_window_create` (fires once at window creation time) and `on_bufferline_mutate` (fires per-line during buffer population). Neither fires when window meta-information changes during navigation — specifically, when the preview target changes from a directory to a file or vice versa, or when navigation changes viewport assignments.

The directory-icons plugin currently sets `prefix_column_width = 2` on all three panes (parent, current, preview) unconditionally in `on_window_create`. This wastes space when the preview shows file content, since file previews have no icons in the prefix column.

The Lua hook bootstrap in `yeet-lua/src/lib.rs` creates hook objects with a shared metatable that provides `:add()`. Hook invocation in `yeet-lua/src/hook.rs` follows a pattern: build context table → iterate callbacks → read back viewport settings.

The central `update::model()` function in `mod.rs` processes all messages for an update cycle, then finalizes window layout and buffer updates. This is the single entry point for all state mutations.

## Goals / Non-Goals

**Goals:**
- Add `on_window_change` hook that fires once per update cycle after all viewport mutations are complete
- Reuse the same context structure as `on_window_create` (type, path, parent/current/preview viewports)
- Ensure the hook fires for all viewport changes: navigation, cursor movement, preview changes, enumeration, path add/remove, resize, etc.
- Enable the directory-icons plugin to conditionally set `prefix_column_width = 2` on preview only when the preview target is a directory

**Non-Goals:**
- Firing `on_window_change` for non-Directory window types (Help, QuickFix, Tasks do not have dynamic meta-information changes)
- Adding new fields to the context table beyond what `on_window_create` already provides
- Adding a generic "what changed" descriptor to the context — plugins inspect current state, not diffs

## Decisions

### Decision 1: Hook fires once at the end of update::model() after all mutations are complete

The `on_window_change` hook fires at the end of `update::model()` after all message processing, window layout finalization, and buffer updates are complete. This ensures:
1. All viewport mutations from the current update cycle are finalized
2. The hook fires exactly once per update cycle, regardless of how many messages were processed
3. The hook captures all types of viewport changes (navigation, cursor movement, preview changes, resize, etc.)

**Alternative**: Fire from `preview::set_buffer_id` only when preview changes. Rejected because it misses navigation events like `navigate::parent` (which swaps viewports without going through preview refresh), and requires threading the Lua runtime through many intermediate call chains.

**Alternative**: Fire on any viewport field change. Rejected because it creates complexity with no current use case and risks performance overhead from multiple invocations per cycle.

### Decision 2: Reuse `build_context` and `read_back_context` from on_window_create

The `on_window_change` invocation reuses the same `build_context` and `read_back_context` functions from the existing `on_window_create` implementation. The context table has the same structure: `{ type = "directory", path = "<current_path>", parent = {...}, current = {...}, preview = {...} }`.

**Alternative**: Create a separate context builder with "what changed" metadata. Rejected — plugins should inspect current state and decide based on that.

### Decision 3: Cycle prevention via end-of-cycle invocation

Cycle prevention is achieved architecturally: the hook fires at the end of `update::model()`, after all message processing is complete. The hook can modify viewport settings (like `prefix_column_width`), but these modifications do not trigger additional messages or re-invoke `update::model()`. There is no re-entrancy path because the hook runs after the message processing loop has completed.

This is inherent cycle prevention — no additional guard flag is needed.

### Decision 4: No Lua runtime threading through internal call chains

Since the hook fires at the end of `update::model()` where `model.lua` is already available, there is no need to thread `Option<&LuaConfiguration>` through intermediate functions like `preview::set_buffer_id`, `selection::*`, `cursor::relocate`, `viewport::relocate`, or `navigate::*`. This keeps the internal function signatures clean.

### Decision 5: Plugin changes preview prefix_column_width in on_window_change

The directory-icons plugin registers an `on_window_change` callback that checks whether the preview buffer is a directory (via `ctx.preview_is_directory`). The plugin sets `ctx.preview.prefix_column_width = 2` when the preview target is a directory, and `ctx.preview.prefix_column_width = 0` when it is not.

The `on_window_create` hook continues to set `prefix_column_width = 2` on parent and current (since those always show directories), but no longer sets it on preview.

### Decision 6: Context includes preview_is_directory flag

To enable plugins to determine whether the preview is showing a directory without filesystem access from Lua, the context table includes a `preview_is_directory` boolean field. This is derived from the preview buffer type in the application state. This avoids the need for Lua-side path inspection or filesystem access.

## Risks / Trade-offs

- [Performance] Hook fires on every update cycle (every cursor movement, every message). Mitigation: Lua callback execution is lightweight; the context table creation is cheap since it reuses existing functions. The hook fires exactly once per cycle.
- [Plugin compatibility] Existing `on_window_create` plugins that set preview `prefix_column_width` will still work — their setting just gets overridden by `on_window_change` on the next update cycle. This is the desired behavior.
- [API surface] Adding a new hook increases the API surface. Mitigation: The hook follows the exact same pattern as `on_window_create`, minimizing cognitive overhead.
