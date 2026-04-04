## Why

`render_window`, `render_buffer_slot`, and `render_directory_buffer` all accept a `buffer_theme: &yeet_buffer::BufferTheme` parameter that is threaded from the top-level `buffer::view()`. These functions already receive `&Theme` (or can receive it). Threading a second theme object adds parameter noise and couples intermediate layout functions to buffer-level rendering details. Each leaf function that actually calls `yeet_buffer::view()` should create the `BufferTheme` locally from `&Theme`.

## What Changes

- Remove `buffer_theme` parameter from `render_window`, `render_buffer_slot`, and `render_directory_buffer` in `yeet-frontend/src/view/buffer.rs`.
- Add `theme: &Theme` parameter to `render_buffer_slot` and `render_directory_buffer` (which currently lack it).
- Create `BufferTheme` via `theme.to_buffer_theme()` at each leaf call site that invokes `yeet_buffer::view()`.
- Remove the `let buffer_theme = theme.to_buffer_theme();` line from the top-level `buffer::view()`.

## Capabilities

### New Capabilities

_(none — this is a refactor of existing internal wiring)_

### Modified Capabilities

_(none — no spec-level behavior changes, only internal parameter threading)_

## Impact

- **yeet-frontend/src/view/buffer.rs**: Internal refactor only. All affected functions are private (`render_window`, `render_buffer_slot`, `render_directory_buffer`). No public API change.
- **Behavior**: Identical rendering output. `to_buffer_theme()` is called per-slot instead of once per frame, but the conversion is trivially cheap (string formatting).
