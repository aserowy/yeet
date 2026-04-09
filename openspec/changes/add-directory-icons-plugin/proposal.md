## Why

Directory buffers currently render plain filenames only, which makes it harder to scan mixed-language trees quickly. Adding a directory-icons plugin with a dedicated icon column and theme-aware colors improves visual parsing while preserving predictable cursor and editing behavior.

## What Changes

- Integrate with the `yeet-directory-icons` plugin through the existing user plugin configuration/loading path (no repository vendoring or plugin-manager workflow changes).
- Add a dedicated, non-editable icon-column segment to shared `@yeet-buffer` prefix rendering for all buffers. A directory window (composed of three `@yeet-buffer` instances) uses this shared function to draw icon content between line numbers and filenames. The column is prefix-only UI and is not part of the underlying editable buffer text content.
- Define icon-column length in `@yeet-buffer` with a strict default of `0` characters. When `yeet-directory-icons` is loaded and executed, it uses an `on_window_create` hook to set icon-column length to `1` character.
- Introduce new hooks in the existing message handling for `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded`. These hooks provide the **complete bufferline and the given window with all metadata** to the plugin at the same point where directory content is set or updated. The plugin **directly mutates each bufferline**: it adds/replaces the icon in the icon column and colors the bufferline text. All icon identification and text color logic lives entirely in the plugin. Deferred `PathsAdded` events (Insert mode) also defer hook invocation; hooks fire on flush.
- Keep cursor/edit operations anchored to filename text: entering Normal/Insert mode does not allow editing the icon column, and cursor start position remains on filenames.
- Remove existing built-in file/directory colorization in directory buffers; the plugin is the single source of truth for icon and text styling.
- The plugin maintains its own easy-to-extend configuration list for icon/style mappings, including defaults for directories (`.direnv`, `target`, `.git`, `.github`) and defaults for filenames that have known Nerd Font icons.
- Default icon colors follow the original Nerd Font icon color palette, and default filename text uses the same mapped base color per rule. This mapping applies to all matching entries (for example all `*.rs` files use the rust icon + rust base color), unless overridden by theme/configuration.

## Capabilities

### New Capabilities
- `directory-icons-plugin`: Directory buffer icon rendering via an optionally installed plugin. The plugin owns all icon identification and text color logic; the core provides mutation hooks with full bufferline and window context so the plugin can directly set icon glyph and styling on each entry. Token names are plugin-defined; directories have their own distinct icon token separate from the file default.

### Modified Capabilities
- `buffer`: `@yeet-buffer` shared prefix model expands with a non-editable icon-column segment for all buffer definitions, including default-width behavior (`0` by default; `1` set via plugin `on_window_create` hook). New mutation hooks in `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling provide complete bufferline and window context, and the plugin directly mutates each bufferline to set icon and text color.
- `theming`: Theme token surface expands to support plugin-provided directory icon color tokens. Token names are plugin-defined; the plugin applies colors by directly mutating bufferlines via hooks.
- `plugins`: Existing plugin loading is reused to consume `yeet-directory-icons` when configured by the user (no plugin-manager changes required). New mutation hooks are introduced in `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling, passing full bufferline and window context to the plugin.

## Impact

- Affects shared `@yeet-buffer` prefix schema, directory rendering pipeline, prefix width calculations, and cursor column mapping.
- Introduces new mutation hook surface in `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling; plugins directly mutate bufferlines to set icon and text color.
- Affects theme token registration and Lua-facing configuration for icon/text class colors (token names are plugin-defined).
- Relies on optional user-installed `yeet-directory-icons`; behavior remains valid with plugin absent (no icons, default styling).
- Plugin directly mutates bufferlines during hook execution, so integration is at the content-lifecycle level rather than rendering level.
