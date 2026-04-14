## Why

The `on_window_change` hook currently exposes a top-level `preview_is_directory` boolean to indicate the preview buffer type, but provides no equivalent type information for the parent or current subtables. This is inconsistent and limits plugin flexibility — plugins cannot determine the buffer type of any viewport subtable in a uniform way. Adding a `buffer_type` property to each subtable (parent, current, preview) makes the context self-describing and allows `preview_is_directory` to be replaced by a generic type check (`ctx.preview.buffer_type == "directory"`).

## What Changes

- Add a `buffer_type` string property to each viewport subtable (`parent`, `current`, `preview`) in the `on_window_change` context table, set to the underlying buffer's type (e.g., `"directory"`, `"content"`, `"image"`, `"empty"`)
- **BREAKING**: Deprecate and remove the top-level `preview_is_directory` boolean from the `on_window_change` context table — replaced by `ctx.preview.buffer_type == "directory"`
- Update the `directory-icons` plugin to use `ctx.preview.buffer_type == "directory"` instead of `ctx.preview_is_directory`
- Update Rust hook invocation code to resolve buffer types for all three viewports and inject them into the context subtables

## Capabilities

### New Capabilities
<!-- None — this change modifies existing capabilities -->

### Modified Capabilities
- `lua`: The `on_window_change` context table gains per-viewport `buffer_type` properties and removes `preview_is_directory`
- `plugins/directory-icons`: The `on_window_change` handler uses `ctx.preview.buffer_type` instead of `ctx.preview_is_directory`

## Impact

- **Rust code**: `yeet-lua/src/hook.rs` — `invoke_on_window_change` and `try_invoke_on_window_change` signatures change to accept buffer types for all three viewports instead of a single `preview_is_directory` bool
- **Rust code**: `yeet-frontend/src/update/hook.rs` — `invoke_on_window_change_for_focused` resolves buffer types for parent, current, and preview buffers
- **Lua plugin**: `plugins/directory-icons/init.lua` — `on_window_change` callback uses `ctx.preview.buffer_type` check
- **Tests**: Rust tests in `yeet-lua/src/hook.rs` and `yeet-frontend/src/update/hook.rs` need updating for new signatures
- **Documentation**: `docs/help/hooks.md` references to `preview_is_directory` need updating
- **Specs**: `openspec/specs/lua/spec.md` and `openspec/specs/plugins/directory-icons/spec.md` need delta specs
