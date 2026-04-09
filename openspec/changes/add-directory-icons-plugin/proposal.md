## Why

Directory buffers currently render plain filenames only, which makes it harder to scan mixed-language trees quickly. Adding a directory-icons plugin with a dedicated icon column and theme-aware colors improves visual parsing while preserving predictable cursor and editing behavior.

## What Changes

- Add a new `yeet-directory-icons` plugin that is vendored as a git submodule at `./plugins/directory-icons` from `git@github.com:aserowy/yeet-directory-icons.git` (submodule folder intentionally omits the `yeet-` prefix).
- Render a dedicated, non-editable icon column in directory buffers between line numbers and filenames; this column is prefix-only UI and is not part of the underlying editable buffer text content.
- Keep cursor/edit operations anchored to filename text: entering Normal/Insert mode does not allow editing the icon column, and cursor start position remains on filenames.
- Support general color styling for both icon glyph and filename text, with matching based on filename extension, exact filename, or directory name.
- Provide one easy-to-extend configuration list for icon/style mappings, including defaults for directories (`.direnv`, `target`, `.git`, `.github`) and defaults for filenames that have known Nerd Font icons.
- Default icon colors follow the original Nerd Font icon color palette unless overridden by theme/configuration.

## Capabilities

### New Capabilities
- `directory-icons-plugin`: Directory buffer icon rendering via a vendored plugin, including icon lookup, icon column drawing, and non-editable icon prefix behavior.

### Modified Capabilities
- `buffer`: Directory buffer layout and cursor semantics change to include a non-editable icon column between line numbers and filenames.
- `theming`: Theme token surface expands to support plugin-provided directory icon color tokens.
- `plugins`: Repository plugin integration includes vendoring and loading the directory-icons plugin from a submodule path.

## Impact

- Affects directory buffer rendering pipeline, prefix width calculations, and cursor column mapping.
- Affects theme token registration and Lua-facing configuration for icon-related colors.
- Adds and tracks a new git submodule under `plugins/directory-icons`.
- Requires integration points between plugin-provided icon metadata and frontend buffer rendering.
