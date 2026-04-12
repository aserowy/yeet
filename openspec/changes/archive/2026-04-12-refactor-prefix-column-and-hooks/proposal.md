## Why

The current buffer model has a separate icon column concept that is tightly coupled to the directory-icons plugin, while the prefix system remains underspecified. Icons should be handled through the prefix column instead, making the model more general. Additionally, several correctness and maintainability issues need fixing: hook firing order relative to signs, help page resolution coupling, plugin-specific documentation leaking into core help docs, and prefix width not being configurable per-buffer.

## What Changes

- **BREAKING**: Remove `icon_column_width` from `ViewPort` and the `icon` field from `BufferLine`; replace with a configurable `prefix_column_width` that plugins use to render icons via the `prefix` field
- Rename `icon_column_width`/`icon_column_length` references to `prefix_column_width` throughout the codebase (Rust, Lua, tests, docs)
- Make `prefix_column_width` configurable per-buffer, defaulting to `0` for all buffers and `1` for the commandline buffer
- Set prefix text alignment to right-aligned within the prefix column
- In `yeet-directory-icons`, set the default prefix column width to `2` (nerd font icons occupy more than one cell)
- Remove plugin-specific documentation from `docs/help/hooks.md` (e.g., "Strip the trailing slash before performing filename-based icon resolution")
- Move help page path resolution (currently hardcoded in `update/command/help.rs`) into the plugin spec so that plugin help pages are resolved at spec initialization time in `yeet-plugin` (Rust side, not Lua)
- **Fix**: Ensure all `on_bufferline_mutate` hooks fire AFTER signs are added (currently hooks fire before signs in `update/enumeration.rs` and potentially other locations)

## Capabilities

### New Capabilities

### Modified Capabilities
- `buffer`: Replace icon column with configurable prefix column; remove `icon`/`icon_column_width` fields; add `prefix_column_width` with per-buffer defaults; right-align prefix text
- `directory-icons-plugin`: Use `prefix` field instead of `icon` field; set prefix column width to `2`; adapt hook handlers for new prefix-based rendering
- `plugins`: Move help page path list into `PluginSpec` so plugin help pages are resolved during spec initialization in `yeet-plugin`
- `help`: Remove plugin-specific documentation from core help pages; update hooks help page to remove directory-icons-specific guidance

## Impact

- **yeet-buffer**: `ViewPort` struct changes (`icon_column_width` → `prefix_column_width`), `BufferLine` struct changes (remove `icon` field), prefix rendering refactored, alignment logic added
- **yeet-lua**: Lua viewport bridge updates (field rename), hook context changes (remove `icon` from mutable fields), viewport Lua table updates
- **yeet-frontend**: Hook invocation order fixed in `update/enumeration.rs` and all other locations where `on_bufferline_mutate` fires; help page resolution moved out of `update/command/help.rs`
- **yeet-plugin**: `PluginSpec` extended with help page paths; help page discovery moved here from frontend
- **plugins/directory-icons**: Rewrite hook handlers to use `prefix` instead of `icon`; set prefix width to `2`; update `on_window_create` hook
- **docs/help/hooks.md**: Remove plugin-specific content
- **Existing plugins**: Any plugin using `icon` field or `icon_column_width` must migrate to `prefix`/`prefix_column_width`
