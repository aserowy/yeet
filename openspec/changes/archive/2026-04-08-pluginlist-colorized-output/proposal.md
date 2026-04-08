## Why

The `:pluginlist` output currently uses `PrintContent::Default` for loaded plugins and `PrintContent::Information` for error/missing plugins. This gives a single green color for both error and missing states. Plugins should be colorized by status — loaded (success), missing (warning), error (error) — using theme-configurable colors.

## What Changes

- Add four new theme tokens: `ErrorFg`, `WarningFg`, `SuccessFg`, `InformationFg`
- Add `PrintContent::Warning` and `PrintContent::Success` variants to complement existing `Error`, `Default`, `Information`
- Update the commandline print rendering to use theme colors for all semantic variants: `ErrorFg` for Error, `WarningFg` for Warning, `SuccessFg` for Success, `InformationFg` for Information
- Update `:pluginlist` output to use `Success` for loaded, `Warning` for missing, `Error` for error

## Capabilities

### New Capabilities

_None_

### Modified Capabilities

- `theming`: Four new color tokens (`ErrorFg`, `WarningFg`, `SuccessFg`, `InformationFg`) added to the theme token set
- `plugins`: `:pluginlist` output uses status-appropriate colors

## Impact

- **yeet-keymap**: `PrintContent` enum gains `Warning` and `Success` variants
- **yeet-frontend**: Theme gets 4 new tokens with defaults; commandline print rendering uses theme ANSI accessors; plugin_list print function uses `Success` for loaded plugins
