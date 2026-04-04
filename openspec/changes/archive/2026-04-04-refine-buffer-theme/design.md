## Context

`BufferTheme` was introduced as a bridge between the frontend theme registry and the buffer crate's rendering. It currently holds 12 fields with mixed types: 9 ANSI escape code strings, 2 `ratatui::style::Color` values, and 1 unused ANSI string (`border_fg`). Five of the string fields are compile-time constants (cursor mode codes and reset sequences) that never change regardless of theme configuration. The `from_enumeration` function receives two pre-computed ANSI strings instead of the theme itself, which couples the caller to knowing exactly which tokens to extract.

## Goals / Non-Goals

**Goals:**
- `BufferTheme` contains only `ratatui::style::Color` fields — one type, no mixing
- Compile-time constants (reset codes, cursor mode codes) are removed from `BufferTheme` and defined as constants in the buffer view module
- The unused `border_fg` ANSI string field is removed
- `from_enumeration` accepts `&Theme` to allow future extensions without signature changes
- Unused token constants are removed from the registry: `COMMANDLINE_FG`, `COMMANDLINE_BG` (registered with defaults but never read in any view code), `CURSOR_NORMAL`, `CURSOR_INSERT` (defined but never registered or used), and `SYNTAX_THEME` (defined but never referenced)

**Non-Goals:**
- Changing the buffer crate's ANSI-based line rendering pipeline to use ratatui `Style` objects
- Adding new theme tokens (that was the previous change)
- Changing the `Ansi` struct or its API

## Decisions

### All BufferTheme fields become `ratatui::style::Color`

The struct's 6 true theme-derived color fields (`cursor_line_bg`, `search_bg`, `line_nr`, `cur_line_nr_bold`, `border_fg_color`, `border_bg_color`) all map to `Color`. The 5 constant fields (`cursor_line_reset`, `cursor_normal_code`, `cursor_normal_reset`, `cursor_insert_code`, `cursor_insert_reset`) are mode/escape codes that don't depend on the theme — they become `const` strings in the buffer view module. The unused `border_fg` is deleted.

**Resulting struct:**
```rust
pub struct BufferTheme {
    pub cursor_line_bg: Color,
    pub search_bg: Color,
    pub line_nr: Color,
    pub cur_line_nr: Color,
    pub border_fg: Color,
    pub border_bg: Color,
}
```

Note: `cur_line_nr_bold` becomes `cur_line_nr` — the bold modifier is a constant applied alongside the color, not theme-configurable.

**Alternative considered:** Converting everything to `String` (ANSI). Rejected because the border rendering uses `Block::border_style()` which requires `Color`, and parsing ANSI strings back to `Color` is fragile.

**Alternative considered:** Converting everything to `ratatui::style::Style`. Rejected because `Style` includes modifiers (bold, underline) that aren't independently theme-configurable for these fields, adding unnecessary complexity.

### Color-to-ANSI conversion in the buffer view

The buffer view's line rendering pipeline builds ANSI-encoded strings via the `Ansi` struct. With `BufferTheme` now holding `Color` values, the view needs to convert them to ANSI escape codes. Two small helper functions (`color_to_ansi_fg`, `color_to_ansi_bg`) are added to the buffer view module.

These duplicates the existing functions in `yeet-frontend/src/theme.rs`. A shared utility crate would eliminate duplication but is out of scope — the buffer crate intentionally has no frontend dependency.

**Alternative considered:** Re-exporting the conversion functions from a shared crate. Out of scope for this change; duplication is acceptable for 2 small pure functions.

### `from_enumeration` receives `&Theme`

Instead of two ANSI strings (`file_fg_ansi`, `directory_fg_ansi`), `from_enumeration` takes `&Theme`. This lets future changes add per-entry styling (e.g., symlink colors, executable colors) without changing the function signature. The function extracts the tokens it needs internally.

This adds a compile-time dependency from the function to `Theme`, but `enumeration.rs` already imports theme types.

### Remove unused token constants

Five token constants in the `tokens` module are dead code:

| Constant | Issue |
|----------|-------|
| `COMMANDLINE_FG` | Registered with default `Color::White` but never used in any view |
| `COMMANDLINE_BG` | Registered with default `Color::Black` but never used in any view |
| `CURSOR_NORMAL` | Defined but never registered in `Default` impl and never used |
| `CURSOR_INSERT` | Defined but never registered in `Default` impl and never used |
| `SYNTAX_THEME` | Defined but never referenced anywhere |

These are removed along with their default color registrations (for `COMMANDLINE_FG`/`COMMANDLINE_BG`). The commandline view currently uses no theme tokens — when it needs them in the future, the constants and defaults should be re-added at that time.

Note: `COMMANDLINE_FG`/`COMMANDLINE_BG` tokens may be set by users in `init.lua`. Removing them means those user settings become silently ignored. This is acceptable since the tokens have no effect today.

## Risks / Trade-offs

- **ANSI conversion duplication** → Two copies of `color_to_ansi_fg`/`color_to_ansi_bg` exist (frontend and buffer crate). Acceptable: they're small, pure, and the buffer crate should not depend on the frontend.
- **Callers must update** → All `BufferTheme` construction sites and all `from_enumeration` call sites change. These were all recently created in the previous change and are well-known.
- **`cur_line_nr_bold` rename** → Renamed to `cur_line_nr` since bold is now a constant. Tests referencing the old name must update.
