## Context

The theme system was recently introduced with token-based color configuration. Most UI elements now use theme tokens, but several gaps remain: file and directory foreground colors in buffers use hardcoded values (or no styling), file permissions in the statusline have no styling, the statusline border lacks a background token, directory window borders and split borders have no dedicated tokens, and the existing `BorderFg` token is defined but not wired through to `yeet-buffer` rendering (which hardcodes `Color::Black`).

## Goals / Non-Goals

**Goals:**
- Every visible color in the UI is customizable through a theme token
- New tokens follow existing naming conventions (`ComponentVariantFg`/`Bg`)
- Buffer entry colors (file/directory) are applied via the theme rather than hardcoded ANSI codes
- The `yeet-buffer` border rendering uses the `BufferTheme.border_fg` field instead of hardcoded black
- Directory window and split borders get their own token pair (fg + bg) for independent customization

**Non-Goals:**
- Per-file-type coloring (e.g., different colors for `.rs` vs `.toml` files)
- Separate tokens for each permission character (r/w/x)
- Background colors for buffer content entries (only foreground is added)

## Decisions

### Token naming and grouping

New tokens follow the established `ComponentVariantProperty` pattern:

| Token | Default | Purpose |
|-------|---------|---------|
| `BufferFileFg` | `White` | File entry foreground in directory buffers |
| `BufferDirectoryFg` | `LightBlue` | Directory entry foreground in directory buffers |
| `StatusLinePermissionsFg` | `Gray` | Permissions text in statusline |
| `StatusLineBorderBg` | `Black` | Background of statusline border area |
| `DirectoryBorderFg` | `Black` | Border fg inside directory window panes |
| `DirectoryBorderBg` | `Reset` | Border bg inside directory window panes |
| `SplitBorderFg` | `Black` | Border fg for vertical split separators (renamed from `BorderFg`) |
| `SplitBorderBg` | `Reset` | Border bg for vertical split separators |

**Rationale:** `DirectoryBorder*` and `SplitBorder*` are separate tokens because users may want different border colors for the three-pane directory view versus split separators. The existing `BorderFg` is renamed to `SplitBorderFg` since it was already used for split/buffer borders.

### File/directory foreground injection via theme

Currently, directory entries are styled with a hardcoded ANSI code in `enumeration.rs`. The change replaces this with a theme-derived ANSI code. File entries currently have no foreground color — they get a new token so users can style them distinctly from regular text content.

**Approach:** The `from_enumeration` function in `enumeration.rs` receives the theme (or pre-computed ANSI strings) and wraps file/directory content with the appropriate ANSI fg code. This keeps the buffer content as ANSI-styled strings, consistent with the existing rendering pipeline.

**Alternative considered:** Passing ratatui `Style` objects alongside buffer lines. Rejected because the buffer crate works with ANSI-encoded strings, and mixing both styling approaches would add complexity.

### Border rendering in yeet-buffer

The `yeet-buffer` view currently hardcodes `Style::default().fg(Color::Black)` for borders. The fix is straightforward: use the existing `theme.border_fg` ANSI value via the `BufferTheme` struct. However, since the buffer view uses ratatui `Block` widgets (not ANSI strings) for borders, the `BufferTheme` also needs a ratatui `Color` or `Style` for the border.

**Approach:** Rename `border_fg` in `BufferTheme` to reflect the split/directory border distinction. Add `border_fg_color` and `border_bg_color` fields as `ratatui::Color` values that the buffer view can use directly with `Block::border_style()`. The caller (frontend) populates these with either `SplitBorderFg`/`SplitBorderBg` or `DirectoryBorderFg`/`DirectoryBorderBg` depending on context.

**Alternative considered:** Parse the ANSI string back to a Color. Rejected as unnecessarily fragile.

### Directory window vs split border distinction

Directory windows (parent/current/preview) use `show_border` on their viewports to draw right-side borders. Split separators also use `show_border`. To distinguish them, the `RenderContext` in `buffer.rs` carries border token information. Directory windows pass `DirectoryBorder*` tokens; vertical splits pass `SplitBorder*` tokens. The buffer view receives the resolved colors through `BufferTheme`.

## Risks / Trade-offs

- **BufferTheme struct growth** → Adding 2 new `Color` fields is minimal. The struct is constructed once per render and passed by reference.
- **Enumeration function signature change** → `from_enumeration` needs access to theme colors, requiring a parameter addition. This changes call sites in `enumeration.rs` but they all have theme access nearby.
- **Default color matching** → New defaults must match the current hardcoded appearance. `LightBlue` for directories matches the current `\x1b[94m`, `White` for files matches the default terminal fg.
