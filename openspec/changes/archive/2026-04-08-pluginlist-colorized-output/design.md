## Context

The commandline print system uses `PrintContent` variants (`Error`, `Default`, `Information`) with hardcoded ANSI color codes. The theme system already has ANSI escape accessors (`Theme::ansi_fg()`) but these aren't used in commandline rendering.

## Goals / Non-Goals

**Goals:**

- Theme tokens `ErrorFg`, `WarningFg`, `SuccessFg`, `InformationFg` with sensible defaults
- `PrintContent::Warning` and `PrintContent::Success` variants with distinct semantics
- Commandline rendering uses theme ANSI colors instead of hardcoded codes
- `:pluginlist` uses status-appropriate colors

**Non-Goals:**

- Background color tokens for these states

## Decisions

### 1. Four new tokens with defaults

Add to `tokens` module:
- `ERROR_FG = "ErrorFg"` — default red (`#ff5555`)
- `WARNING_FG = "WarningFg"` — default yellow (`#f1fa8c`)
- `SUCCESS_FG = "SuccessFg"` — default green (`#50fa7b`)
- `INFORMATION_FG = "InformationFg"` — default blue (`#8be9fd`)

### 2. PrintContent::Warning and PrintContent::Success variants

Add `Warning(String)` and `Success(String)` to the `PrintContent` enum in `yeet-keymap`. Each variant has clear semantics:
- `Error` = error (red)
- `Warning` = warning (yellow)
- `Success` = success/confirmation (green)
- `Information` = neutral informational (blue)
- `Default` = unstyled

### 3. Theme-aware commandline rendering

The `commandline::print` function accepts `&Theme` and uses theme ANSI accessors for all semantic variants. `Default` remains unstyled.

### 4. Pluginlist color mapping

- `loaded` → `PrintContent::Success` (green)
- `missing` → `PrintContent::Warning` (yellow)
- `error` → `PrintContent::Error` (red)
