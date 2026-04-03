## Context

In `yeet-frontend/src/view/buffer.rs`, the top-level `view()` creates a `BufferTheme` and threads it through three levels of private functions: `render_window` → `render_buffer_slot` → `render_directory_buffer`. All these functions already receive `&Theme` or can trivially accept it. The `BufferTheme` is only consumed at the leaf level by calls to `yeet_buffer::view()`.

## Goals / Non-Goals

**Goals:**
- Remove `buffer_theme` parameter from all intermediate private functions.
- Create `BufferTheme` at each leaf call site via `theme.to_buffer_theme()`.
- Pass `&Theme` to `render_buffer_slot` and `render_directory_buffer` instead.

**Non-Goals:**
- Caching or memoizing `to_buffer_theme()` — the conversion is trivially cheap.
- Changing `commandline.rs` — it already creates `BufferTheme` locally.
- Modifying any public API or cross-crate signatures.

## Decisions

### Convert at leaf call sites, not intermediate functions

`render_window` is a recursive layout function that doesn't need buffer-level theme details. Only `render_buffer_slot` and `render_directory_buffer` call `yeet_buffer::view()`, so only they need to create the `BufferTheme`. This keeps the layout layer decoupled from rendering details.

### Accept multiple `to_buffer_theme()` calls per frame

The conversion allocates a few small strings. With at most ~5 buffer slots visible, this is negligible. Optimizing with a single cached conversion would reintroduce the parameter threading we're removing.

## Risks / Trade-offs

- **[Slightly more allocations per frame]** → Negligible: `to_buffer_theme()` creates ~10 small strings, called at most a handful of times per render. No measurable impact.
