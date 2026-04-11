## Why

Directory buffers currently render plain filenames only, which makes it harder to scan mixed-language trees quickly. Adding a directory-icons plugin with a dedicated icon column and theme-aware colors improves visual parsing while preserving predictable cursor and editing behavior.

## What Changes

- Integrate with the `yeet-directory-icons` plugin through the existing user plugin configuration/loading path (no repository vendoring or plugin-manager workflow changes).
- Add a dedicated, non-editable icon-column segment to shared `@yeet-buffer` prefix rendering for all buffers. A directory window (composed of three `@yeet-buffer` instances) uses this shared function to draw icon content between line numbers and filenames. The column is prefix-only UI and is not part of the underlying editable buffer text content.
- Define icon-column length in `@yeet-buffer` with a strict default of `0` characters. When `yeet-directory-icons` is loaded and executed, it uses an `on_window_create` hook to set icon-column length to `1` character.
- Expand the `on_bufferline_mutate` hook to provide the **complete bufferline** together with **wrapping buffer metadata** as context. The buffer metadata identifies the buffer type (using the `Buffer` enum — e.g., `Directory`, `Content`, `Help`, `QuickFix`, `Tasks`) so the plugin knows which buffer it is processing. Directory buffers include the parent path; content buffers include the file path. The hook fires for **all buffer types**, not just directory buffers.
- The plugin is responsible for checking the buffer type in each hook invocation and deciding whether to act (e.g., only mutate file/directory-related buffers).
- Inside the hook, the entire bufferline (excluding line numbers) is mutable: `prefix`, `content` (Ansi), `search_char_position`, `signs`, and `icon`. The plugin directly mutates these fields in-place.
- Remove the `icon_style` field from `BufferLine`. The plugin handles all content styling by mutating the `content` Ansi string directly (prepending ANSI escape sequences). The core does not apply any icon-related styling — it only renders the mutated content as-is.
- Remove existing built-in file/directory colorization in directory buffers with no fallback; the plugin is the single source of truth for icon and text styling. Without the plugin, entries render as plain unstyled text.
- Directory names always end with a trailing slash (`/`) in the bufferline content so that the user and the plugin can differentiate between filenames and directory names without needing a separate `is_directory` flag.
- After adopting the trailing-slash naming convention, the `ContentKind` enum and the `is_directory` hook parameter are removed since directory-ness is now encoded in the name itself.
- The plugin maintains its own easy-to-extend configuration list for icon/style mappings, including defaults for directories (`.direnv/`, `target/`, `.git/`, `.github/`) and defaults for filenames that have known Nerd Font icons.
- Default icon colors follow the original Nerd Font icon color palette, and default filename text uses the same mapped base color per rule. This mapping applies to all matching entries (for example all `*.rs` files use the rust icon + rust base color), unless overridden by theme/configuration.
- Theme tokens set by the directory-icons plugin can be overwritten by theme plugins (e.g., `yeet-bluloco-theme`). When a theme plugin sets a theme token first, `yeet-directory-icons` checks for existing theme values and does not overwrite them.

## Capabilities

### New Capabilities
- `directory-icons-plugin`: Directory buffer icon rendering via an optionally installed plugin. The plugin owns all icon identification and text color logic; the core provides mutation hooks with full bufferline and buffer-type metadata so the plugin can directly set icon glyph and style content on each entry. Token names are plugin-defined; directories have their own distinct icon token separate from the file default.

### Modified Capabilities
- `buffer`: `@yeet-buffer` shared prefix model expands with a non-editable icon-column segment for all buffer definitions, including default-width behavior (`0` by default; `1` set via plugin `on_window_create` hook). The `on_bufferline_mutate` hook now fires for all buffer types and exposes the full bufferline (prefix, content, search_char_position, signs, icon) plus buffer-type metadata. The `icon_style` field is removed; plugins style content by mutating the Ansi string directly. Directory names in bufferline content end with a trailing slash.
- `theming`: Theme token surface expands to support plugin-provided directory icon color tokens. Token names are plugin-defined; the plugin applies colors by directly mutating bufferline content. Theme plugins can override tokens set by the icons plugin.
- `plugins`: Existing plugin loading is reused to consume `yeet-directory-icons` when configured by the user (no plugin-manager changes required). The `on_bufferline_mutate` hook fires for all buffer types, passing full bufferline and buffer-type metadata to the plugin.

## Impact

- Affects shared `@yeet-buffer` prefix schema, directory rendering pipeline, prefix width calculations, and cursor column mapping.
- Introduces expanded mutation hook surface: hooks fire for all buffer types with full bufferline and buffer-type metadata; plugins directly mutate bufferlines to set icon and style content.
- Removes `icon_style` from `BufferLine` and all related core styling logic; plugins own content styling entirely.
- Removes `ContentKind` enum and `is_directory` hook parameter after adopting trailing-slash naming convention for directories.
- Affects theme token registration and Lua-facing configuration for icon/text class colors (token names are plugin-defined, overrideable by theme plugins).
- Relies on optional user-installed `yeet-directory-icons`; behavior remains valid with plugin absent (no icons, plain unstyled text).
- Plugin directly mutates bufferlines during hook execution, so integration is at the content-lifecycle level rather than rendering level.
