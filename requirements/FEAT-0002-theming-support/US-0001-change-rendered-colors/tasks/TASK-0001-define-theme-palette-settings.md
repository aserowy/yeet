# Task: Define Theme Palette Settings

## Metadata

- ID: TASK-0001
- Status: plan
- Userstory: US-0001

## Motivation

We need a single, well-defined palette structure so users can specify rendered UI colors consistently across the app. This provides the foundation for applying user-configurable colors to the UI.

## Relevant Acceptance Criteria

- Given the application supports theming
- When I set new color values for the theme palette
- Then the rendered UI uses the updated colors consistently

## Requirements

- Introduce a theme palette data structure in the frontend settings/model layer that captures the UI colors used by rendered surfaces.
- The palette must include explicit fields for all currently hard-coded UI colors used in view rendering (e.g., tabbar, statusline).
- Provide default palette values that match the current hard-coded colors to preserve existing behavior when no customization is set.
- Expose the palette through `Settings` so it can be read by rendering code.

## Exclusions

- Do NOT implement user input or CLI flags for setting theme values in this task.
- Do NOT change any rendering logic in this task; only introduce the palette model and defaults.
- Do NOT change non-frontend crates.

## Context

- @yeet-frontend/src/settings.rs — current Settings structure and defaults.
- @yeet-frontend/src/view/statusline.rs — uses hard-coded colors that should be represented in the palette.
- @yeet-frontend/src/view/tabbar.rs — uses hard-coded colors that should be represented in the palette.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Add a theme palette struct

Create a palette struct (e.g., `ThemePalette`) with fields for each rendered color currently hard-coded in view components.

```rust
pub struct ThemePalette {
    pub tab_active_bg: Color,
    pub tab_active_fg: Color,
    pub tab_inactive_bg: Color,
    pub tab_inactive_fg: Color,
    pub tab_fill_bg: Color,
    pub statusline_bg: Color,
    pub statusline_fg: Color,
    pub statusline_dim_fg: Color,
    pub statusline_border_fg: Color,
    pub statusline_success_fg: Color,
    pub statusline_warning_fg: Color,
    pub statusline_error_fg: Color,
    // add any other required UI colors used in views
}
```

### Step 2: Add palette to Settings with defaults

Add the palette to `Settings` and initialize defaults to match current constants (e.g., `Color::Black`, `Color::LightBlue`, etc.).

```rust
pub struct Settings {
    pub theme: ThemePalette,
    // existing fields...
}
```

```rust
impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: ThemePalette::default(),
            // existing fields...
        }
    }
}
```

### Step 3: Provide a `Default` implementation for the palette

Use the current hard-coded view colors as defaults so rendering output remains unchanged until customized.

## Examples

- Default palette values match current tabbar/statusline colors.
- No rendering output changes when the palette is introduced.
