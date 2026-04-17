## Why

The recent security fix (commit `79b2ecb` — "fix SVG preview escape causing terminal breakout and RCE") introduced pre-highlight line truncation to prevent pathological multi-megabyte ANSI strings from breaking the terminal. However, this truncation cuts raw source text before passing it to syntect, which corrupts syntect's stateful parser: when a line like `src="https://...long-url..."` is truncated mid-string-literal, the closing `"` is lost, and syntect carries the "inside a string" state into all subsequent lines, causing them to be rendered with string-literal styling.

## What Changes

- Remove the `truncate_line` pre-highlight truncation from `yeet-frontend/src/task/syntax.rs` so syntect receives full original lines and maintains correct parser state
- Keep the `strip_non_sgr_escape_sequences` sanitization (the actual security fix against terminal breakout)
- Rely on the existing view layer (horizontal scrolling, word wrapping) for display-width fitting

## Capabilities

### New Capabilities

### Modified Capabilities

- `rendering`: The preview syntax highlighting pipeline must pass full lines to syntect to preserve cross-line parser state, relying on the view layer for display-width fitting

## Impact

- **Code**: `yeet-frontend/src/task/syntax.rs` — remove `truncate_line()` and `content_width` parameter from `highlight()`
- **Code**: `yeet-frontend/src/task/mod.rs` — update call site to stop passing `rect.width`
- **Dependencies**: No changes; the non-SGR sanitization from the security fix is preserved
