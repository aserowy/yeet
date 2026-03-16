# Task: Apply Palette to View Rendering

## Metadata

- ID: TASK-0002
- Status: done
- Userstory: US-0001

## Motivation

The palette must drive all rendered colors so that setting new theme values actually changes the UI appearance consistently.

## Relevant Acceptance Criteria

- Given the application supports theming
- When I set new color values for the theme palette
- Then the rendered UI uses the updated colors consistently

## Requirements

- Replace hard-coded UI colors in view rendering with references to the theme palette.
- Ensure every UI surface that currently uses hard-coded `Color::` values pulls those values from the palette.
- Thread the palette into view rendering via existing settings plumbing.
- Update any existing tests that assert label formatting or rendering behavior to account for palette-driven styles.

## Exclusions

- Do NOT add new theming surfaces beyond those already rendered.
- Do NOT alter layout or rendering structure; only replace color sources.
- Do NOT change non-frontend crates.

## Context

- @yeet-frontend/src/view/statusline.rs — statusline styles use hard-coded colors.
- @yeet-frontend/src/view/tabbar.rs — tabbar styles use hard-coded colors.
- @yeet-frontend/src/view/window.rs — constructs view rendering with settings.
- @yeet-frontend/src/model/mod.rs — app model references settings.
- @AGENTS.md — build/test/lint commands.

## Implementation Plan

### Step 1: Wire palette into rendering entry points

Ensure rendering functions have access to `Settings` or a `ThemePalette` reference.

### Step 2: Replace hard-coded colors in tabbar

Swap direct `Color::*` usage with palette fields.

```rust
Style::default().bg(settings.theme.tab_active_bg).fg(settings.theme.tab_active_fg)
```

### Step 3: Replace hard-coded colors in statusline

Swap `Color::Black`, `Color::Gray`, `Color::White`, `Color::Green/Yellow/Red`, and border colors with palette fields.

### Step 4: Update tests

Adjust any tests that implicitly depend on hard-coded colors, if needed, to use defaults from the palette.

### Step 5: Run lint/test

- `cargo fmt`
- `cargo clippy --all-targets --all-features`
- `cargo test`

## Examples

- Changing `settings.theme.tab_active_bg` from `LightBlue` to `DarkGray` changes the active tab background color.
- Changing `settings.theme.statusline_bg` updates both focused and unfocused statusline backgrounds consistently.
