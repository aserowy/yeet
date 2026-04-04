## Why

Several UI elements still use hardcoded colors or no styling at all, preventing full theme customization. Users cannot customize the foreground color of files and directories in buffers, file permissions in the statusline, the statusline border background, the border colors inside directory-type windows, or the border colors for split separators. This blocks a cohesive custom theme experience.

## What Changes

- Add `BufferFileFg` token for file entry foreground color in directory buffers
- Add `BufferDirectoryFg` token for directory entry foreground color in directory buffers (replaces hardcoded `\x1b[94m`)
- Add `StatusLinePermissionsFg` token for file permissions text in the statusline
- Add `StatusLineBorderBg` token for the statusline border background
- Add `DirectoryBorderFg` and `DirectoryBorderBg` tokens for borders inside directory-type windows (parent/current/preview panes)
- Rename existing `BorderFg` to `SplitBorderFg` and add `SplitBorderBg` for vertical split separator borders, wiring them through `BufferTheme` into actual buffer rendering (currently hardcoded to `Color::Black` in `yeet-buffer/src/view/mod.rs`)

## Capabilities

### New Capabilities

- `extended-theme-tokens`: Adds new color tokens and wires them through the rendering pipeline so all visible UI colors are theme-configurable

### Modified Capabilities

- `theme-registry`: New tokens must be registered with defaults in the theme registry
- `theme-integration`: New tokens must be integrated into the rendering code for statusline, buffer entries, and border widgets

## Impact

- `yeet-frontend/src/theme.rs`: New token constants and default colors
- `yeet-frontend/src/update/enumeration.rs`: Replace hardcoded ANSI directory color with theme token
- `yeet-frontend/src/view/statusline.rs`: Apply permissions and border background tokens
- `yeet-frontend/src/view/buffer.rs`: Apply directory window and split border tokens
- `yeet-buffer/src/lib.rs`: Extend `BufferTheme` struct with new fields if needed
- `yeet-buffer/src/view/mod.rs`: Use `BufferTheme` border colors instead of hardcoded `Color::Black`
