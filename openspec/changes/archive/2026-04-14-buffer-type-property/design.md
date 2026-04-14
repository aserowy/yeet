## Context

The `on_window_change` hook fires for Directory windows when navigation changes viewport paths or buffers. The context table currently exposes three viewport subtables (`parent`, `current`, `preview`) with viewport settings and paths, plus a top-level `preview_is_directory` boolean. The underlying buffers already have well-defined types (the `Buffer` enum in `yeet-frontend` and the `BufferType` enum in `yeet-lua`), but this type information is only exposed to Lua in the `on_bufferline_mutate` hook — not in `on_window_change`.

The `BufferType` enum in `yeet-lua` covers 5 variants: `directory`, `content`, `help`, `quickfix`, `tasks`. The application `Buffer` enum has additional variants (`Image`, `PathReference`, `Empty`) that don't map 1:1. For `on_window_change`, the parent viewport always holds a `Directory` buffer, the current always holds a `Directory` buffer, and the preview can hold any buffer type.

## Goals / Non-Goals

**Goals:**
- Add a `buffer_type` string property to each viewport subtable in the `on_window_change` context
- Remove the top-level `preview_is_directory` boolean (replaced by `ctx.preview.buffer_type == "directory"`)
- Update the directory-icons plugin to use the new property
- Update documentation and specs
- Keep backward compatibility impact minimal (only `preview_is_directory` removal is breaking)

**Non-Goals:**
- Adding `buffer_type` to `on_window_create` context (those windows are always freshly created with known types)
- Changing the `on_bufferline_mutate` hook's `ctx.buffer.type` field
- Exposing all `Buffer` enum variants to Lua — only the existing `BufferType` values plus `"image"` and `"empty"` for unmapped variants

## Decisions

### Decision 1: Map all Buffer variants to string types for Lua

**Choice**: Extend the type mapping to cover all `Buffer` enum variants that can appear in a directory window's viewports.

**Mapping**:
| Buffer variant | Lua string |
|---|---|
| `Buffer::Directory(_)` | `"directory"` |
| `Buffer::Content(_)` | `"content"` |
| `Buffer::Image(_)` | `"image"` |
| `Buffer::Help(_)` | `"help"` |
| `Buffer::QuickFix(_)` | `"quickfix"` |
| `Buffer::Tasks(_)` | `"tasks"` |
| `Buffer::PathReference(_)` | `"content"` |
| `Buffer::Empty` | `"empty"` |

**Rationale**: `PathReference` is treated as `"content"` because it represents a file path reference used for content preview. This keeps the Lua API simple while covering all cases.

**Alternative considered**: Reuse the existing `BufferType` enum — rejected because it doesn't cover `Image` and `Empty` which are valid preview buffer states.

### Decision 2: Add a `buffer_type_for_lua` method on `Buffer`

**Choice**: Add a method `fn buffer_type_for_lua(&self) -> &'static str` directly on the `Buffer` enum in `yeet-frontend/src/model/mod.rs`.

**Rationale**: This centralizes the mapping logic and avoids duplicating match arms in the hook code. The method lives on `Buffer` because the mapping is about the `Buffer` enum, not the `BufferType` enum (which is specific to `on_bufferline_mutate`).

### Decision 3: Change `invoke_on_window_change` signature

**Choice**: Replace the `preview_is_directory: bool` parameter with `buffer_types: [&str; 3]` representing the buffer types for `[parent, current, preview]`.

**Rationale**: This is more general and forward-looking. The caller already resolves buffer IDs for all three viewports, so determining buffer types for all three is trivial.

### Decision 4: Set `buffer_type` on each subtable in `try_invoke_on_window_change`

**Choice**: After `build_context` creates the subtables, set `buffer_type` on each one (alongside the existing `path` injection loop).

**Rationale**: Minimal change to existing code structure. The `build_context` function remains generic (used by both `on_window_create` and `on_window_change`), and `buffer_type` is only added in the `on_window_change` path.

### Decision 5: Remove `preview_is_directory` entirely (breaking change)

**Choice**: Remove the `preview_is_directory` field from the context table instead of deprecating it alongside the new field.

**Rationale**: The only consumer is the directory-icons plugin which we control. There are no third-party plugins in the ecosystem yet. A clean break avoids redundant fields.

## Risks / Trade-offs

- **[Breaking change]** → Mitigated by updating the only known consumer (directory-icons plugin) in the same change. User configs that use `preview_is_directory` will silently get `nil` — this is acceptable since the replacement is straightforward.
- **[Buffer type may be nil]** → If a viewport has no buffer, `buffer_type` will be `nil`. Plugins must handle this. Mitigated by documenting that `buffer_type` is `nil` when no buffer is assigned.
