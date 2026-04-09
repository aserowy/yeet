## Why

Directory buffers currently render plain filenames only, which makes it harder to scan mixed-language trees quickly. Adding a directory-icons plugin with a dedicated icon column and theme-aware colors improves visual parsing while preserving predictable cursor and editing behavior.

## What Changes

- Integrate with the `yeet-directory-icons` plugin through the existing user plugin configuration/loading path (no repository vendoring or plugin-manager workflow changes).
- Add a dedicated, non-editable icon-column segment to shared `@yeet-buffer` prefix rendering for all buffers. A directory window (composed of three `@yeet-buffer` instances) uses this shared function to draw icons between line numbers and filenames. The column is prefix-only UI and is not part of the underlying editable buffer text content.
- Define icon-column length in `@yeet-buffer` with a strict default of `0` characters. When `yeet-directory-icons` is loaded and executed, it uses an `on_window_create` hook to set icon-column length to `1` character.
- Keep cursor/edit operations anchored to filename text: entering Normal/Insert mode does not allow editing the icon column, and cursor start position remains on filenames.
- Support general color styling for both icon glyph and filename text, with matching based on filename extension, exact filename, or directory name.
- Remove existing built-in file/directory colorization in directory buffers before applying plugin-driven icon/text styling.
- Provide one easy-to-extend configuration list for icon/style mappings, including defaults for directories (`.direnv`, `target`, `.git`, `.github`) and defaults for filenames that have known Nerd Font icons.
- Default icon colors follow the original Nerd Font icon color palette, and default filename text uses the same mapped base color per rule. This mapping applies to all matching entries (for example all `*.rs` files use the rust icon + rust base color), unless overridden by theme/configuration.

## Capabilities

### New Capabilities
- `directory-icons-plugin`: Directory buffer icon rendering via an optionally installed plugin, including icon lookup, icon column drawing, and non-editable icon prefix behavior.

### Modified Capabilities
- `buffer`: `@yeet-buffer` shared prefix model expands with a non-editable icon-column segment for all buffer definitions, including default-width behavior (`0` without plugin, `1` with plugin); directory window rendering populates it between line numbers and filenames through shared buffer rendering.
- `buffer`: `@yeet-buffer` shared prefix model expands with a non-editable icon-column segment for all buffer definitions, including default-width behavior (`0` by default; `1` set via plugin `on_window_create` hook); directory window rendering populates it between line numbers and filenames through shared buffer rendering.
- `theming`: Theme token surface expands to support plugin-provided directory icon color tokens.
- `plugins`: Existing plugin loading is reused to consume `yeet-directory-icons` when configured by the user (no plugin-manager changes required).

## Impact

- Affects shared `@yeet-buffer` prefix schema, directory rendering pipeline, prefix width calculations, and cursor column mapping.
- Affects theme token registration and Lua-facing configuration for icon/text class colors.
- Relies on optional user-installed `yeet-directory-icons`; behavior remains valid with plugin absent.
- Requires integration points between plugin-provided icon metadata and frontend buffer rendering.
