## Context

The `on_window_change` hook currently receives a single top-level `ctx.path` set to the current directory's path. The three viewport subtables (`ctx.parent`, `ctx.current`, `ctx.preview`) only contain viewport settings (line_number, wrap, etc.) with no path information. Plugins that need to know what path each viewport is showing must perform filesystem inspection from Lua, which is not available.

Each viewport's buffer already has a resolvable path via `buffer.resolve_path()`. The parent buffer resolves to the parent directory, the current buffer resolves to the current directory, and the preview buffer resolves to the preview target (directory or file).

## Goals / Non-Goals

**Goals:**
- Add a `path` property to each viewport subtable in the `on_window_change` context (`ctx.parent.path`, `ctx.current.path`, `ctx.preview.path`)
- Remove the top-level `ctx.path` from `on_window_change` since per-viewport paths supersede it
- Keep `on_window_create` context unchanged (it still uses top-level `ctx.path`)

**Non-Goals:**
- Adding per-viewport paths to `on_window_create` (different lifecycle, not needed)
- Making the `path` property writable (read-only informational field)
- Adding path to single-viewport window types (Help, QuickFix, Tasks don't use `on_window_change`)

## Decisions

### Decision 1: Per-viewport path set on viewport subtable after build_context

The `try_invoke_on_window_change` function sets `path` on each viewport subtable after `build_context` creates them, rather than modifying `build_context` itself. This keeps `build_context` unchanged for `on_window_create` and isolates the per-viewport path logic to `on_window_change`.

The function signature changes from `path: Option<&Path>` to accepting three optional paths: `parent_path`, `current_path`, `preview_path`.

**Alternative**: Modify `build_context` to accept per-viewport paths. Rejected because it would complicate the shared function for no benefit to `on_window_create`.

### Decision 2: Top-level ctx.path removed from on_window_change

Since per-viewport paths provide strictly more information, the top-level `ctx.path` is removed from `on_window_change`. This avoids redundancy and makes the API clean.

### Decision 3: Path is read-only (not read back)

The `path` property on viewport subtables is informational only. The `read_back_context` function does not read back `path` values. This is consistent with `preview_is_directory` which is also read-only.

## Risks / Trade-offs

- [Breaking change] Any existing `on_window_change` callbacks that read `ctx.path` will get `nil`. Mitigation: The feature is new (just added), so no external consumers exist yet. The directory-icons plugin does not use `ctx.path`.
- [API inconsistency] `on_window_create` uses top-level `ctx.path`, `on_window_change` uses per-viewport paths. Mitigation: The two hooks have different contexts — create fires once at window creation, change fires per-function with richer viewport state.
