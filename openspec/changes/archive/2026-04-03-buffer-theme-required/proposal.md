## Why

`yeet-buffer` currently owns a `BufferTheme` with a `Default` impl containing hardcoded ANSI codes, plus a convenience `view()` that uses those defaults. This duplicates color knowledge that already lives in `yeet-frontend::theme::Theme` and makes it possible to render buffers without going through the centralized theme system introduced in the `theming` change. Removing the default and the convenience wrapper enforces a single source of truth for all colors.

## What Changes

- **BREAKING**: Remove `Default` impl from `yeet_buffer::BufferTheme`.
- **BREAKING**: Remove `yeet_buffer::view()` (the non-themed convenience function). Only `yeet_buffer::view_themed()` remains — rename it back to `view()` since it becomes the only variant.
- Move the default `BufferTheme` values into `yeet-frontend::theme::Theme::to_buffer_theme()` (already the only real construction site).

## Capabilities

### New Capabilities

- `buffer-theme-injection`: Enforce that buffer rendering always receives an externally-provided `BufferTheme`, eliminating internal defaults in `yeet-buffer`.

### Modified Capabilities

_(none — the theme-registry spec is unaffected; it already defines the tokens and `to_buffer_theme()`)_

## Impact

- **yeet-buffer**: Public API change — `BufferTheme::default()` removed, `view()` removed, `view_themed()` renamed to `view()`.
- **yeet-frontend**: No behavioral change; already passes theme everywhere. Import alias `view_themed as buffer_view` updated to `view as buffer_view`.
- **Downstream crates**: Any crate that calls `yeet_buffer::view()` or relies on `BufferTheme::default()` will fail to compile until updated (compile-time safety, no silent breakage).
