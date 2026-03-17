# Task: Apply Buffer and Border Backgrounds in Rendering

## Metadata

- ID: TASK-0002
- Status: done
- Userstory: US-0003

## Motivation

The palette fields must be consumed by rendering code so buffer surfaces and borders display the configured backgrounds. Without this, the new settings have no effect.

## Relevant Acceptance Criteria

- When I set background colors for buffer content surfaces, miller column borders, and split borders
- Then rendered buffers, miller column borders, and split borders use those configured background colors consistently

## Requirements

- Apply `settings.theme.buffer_bg` to buffer content rendering (directory, tasks, and content buffers).
- Apply `settings.theme.miller_border_bg` to miller column borders in the directory window.
- Apply `settings.theme.split_border_bg` to borders used for split separators.
- Preserve existing foreground/border colors unless needed to keep visuals stable.
- Ensure borders differentiate split borders vs miller column borders even if they share rendering code.

## Exclusions

- Do NOT add new theme fields in this task (handled in TASK-0001).
- Do NOT change tabbar/statusline theming.
- Do NOT change layout or viewport sizing logic.

## Context

- @yeet-frontend/src/view/buffer.rs — orchestrates buffer rendering and knows split vs directory context.
- @yeet-buffer/src/view/mod.rs — renders buffer content and borders with ratatui.
- @yeet-frontend/src/model/mod.rs — `Window::Directory` vs split windows with `show_border`.
- @requirements/FEAT-0002-theming-support/US-0003-configure-buffer-backgrounds/story.md — acceptance criteria.

## Implementation Plan

### Step 1: Thread background values into buffer rendering

Introduce a way to pass background colors into `yeet_buffer::view` (e.g., an added parameter, a small render config struct, or fields on `ViewPort`). The goal is to supply `buffer_bg` and a border background to the view layer.

```rust
pub struct RenderStyles {
    pub buffer_bg: Color,
    pub border_bg: Option<Color>,
}
```

### Step 2: Apply buffer background in yeet-buffer view

Use `Style::default().bg(buffer_bg)` when rendering the `Paragraph` so the buffer surface is filled with the configured background.

### Step 3: Apply border backgrounds by context

In `yeet-frontend/src/view/buffer.rs`, select the border background based on context:

- For directory window columns (parent/current with `show_border=true`), pass `miller_border_bg`.
- For split borders (when `draw_borders: Some(true)`), pass `split_border_bg`.

Then use the provided border background in `yeet-buffer/src/view/mod.rs` when building the `Block` for borders (e.g., `Style::default().bg(border_bg).fg(existing_border_fg)`).

### Step 4: Preserve default behavior

If no background is configured (or if using `Color::Reset`), ensure the rendered output matches today’s visuals.

## Examples

- Setting `buffer_bg` to `#1E1E1E` results in buffer surfaces filled with that color.
- Setting `split_border_bg` to `#333333` only affects split separators, not miller column borders.
