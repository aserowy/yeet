# Task: Define Buffer and Border Palette Fields

## Metadata

- ID: TASK-0001
- Status: plan
- Userstory: US-0003

## Motivation

We need explicit palette fields (and Lua overrides) for buffer backgrounds and border backgrounds so rendering can be configured without changing code. This establishes the source of truth for theming these surfaces.

## Relevant Acceptance Criteria

- When I set background colors for buffer content surfaces, miller column borders, and split borders
- Then rendered buffers, miller column borders, and split borders use those configured background colors consistently

## Requirements

- Add new palette fields for buffer backgrounds, miller column border backgrounds, and split border backgrounds in `ThemePalette`.
- Provide defaults that preserve current appearance (avoid visual changes when not configured).
- Extend Lua theme overrides to accept the new fields under `y.theme`.
- Ensure settings defaults include the new palette values.

## Exclusions

- Do NOT change rendering behavior in this task.
- Do NOT add new UI surfaces beyond the three specified in this story.
- Do NOT change unrelated theme fields.

## Context

- @yeet-frontend/src/settings.rs — `ThemePalette` definition and defaults.
- @yeet-frontend/src/lua_settings.rs — theme override parsing and application.
- @requirements/FEAT-0002-theming-support/US-0003-configure-buffer-backgrounds/story.md — story scope and acceptance criteria.

## Implementation Plan

### Step 1: Add palette fields with defaults

Add fields like `buffer_bg`, `miller_border_bg`, and `split_border_bg` to `ThemePalette` and initialize defaults to preserve current visuals (e.g., `Color::Reset` for buffer background if the terminal default is used today, or the current hard-coded border color if applicable).

```rust
pub struct ThemePalette {
    pub buffer_bg: Color,
    pub miller_border_bg: Color,
    pub split_border_bg: Color,
    // existing fields...
}
```

### Step 2: Extend Lua overrides

Add override keys (e.g., `buffer_bg`, `miller_border_bg`, `split_border_bg`) to `ThemePaletteOverrides` and map them in `read_theme_palette_overrides`.

```rust
apply_color("buffer_bg", &mut overrides.buffer_bg);
apply_color("miller_border_bg", &mut overrides.miller_border_bg);
apply_color("split_border_bg", &mut overrides.split_border_bg);
```

### Step 3: Update defaults wiring

Ensure `Settings::default()` uses the updated `ThemePalette::default()` so the new fields always exist.

## Examples

- Setting `y.theme.buffer_bg = "#1E1E1E"` should update the buffer background color.
- Leaving `y.theme.miller_border_bg` unset should keep the current miller column border appearance.
