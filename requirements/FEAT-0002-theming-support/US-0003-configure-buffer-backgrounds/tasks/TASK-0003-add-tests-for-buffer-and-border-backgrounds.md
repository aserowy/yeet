# Task: Add Tests for Buffer and Border Backgrounds

## Metadata

- ID: TASK-0003
- Status: done
- Userstory: US-0003

## Motivation

Tests ensure background theming works as intended and that defaults remain stable. This prevents regressions in rendering when palette behavior changes.

## Relevant Acceptance Criteria

- Then rendered buffers, miller column borders, and split borders use those configured background colors consistently
- Ensure defaults preserve the current appearance when no configuration is provided

## Requirements

- Add tests that validate buffer background rendering uses the configured palette value.
- Add tests that validate miller column border and split border backgrounds are applied distinctly.
- Add tests or assertions that confirm defaults match the current look (no unintended changes).

## Exclusions

- Do NOT add integration tests that require terminal snapshots or golden images.
- Do NOT modify production rendering behavior in this task.

## Context

- @yeet-frontend/src/view/window.rs — existing render tests for buffers and tabs.
- @yeet-buffer/src/view/mod.rs — border rendering logic.
- @yeet-frontend/src/settings.rs — palette defaults.
- @AGENTS.md — testing commands.

## Implementation Plan

### Step 1: Identify or extend render test utilities

Reuse existing helpers (if any) that render to a `ratatui::buffer::Buffer` and inspect cells for style attributes.

### Step 2: Add buffer background assertions

Render a buffer with a non-default `buffer_bg` and assert that at least one cell in the buffer area uses the configured background color.

### Step 3: Add border background assertions

Render a directory window (miller columns) and a split window; assert that border cells for each use their corresponding background values.

### Step 4: Verify defaults

Ensure a test path with `ThemePalette::default()` still yields the same colors as before (e.g., border backgrounds are `Color::Reset` if that preserves the previous appearance).

## Examples

- `buffer_bg = Color::Rgb(30, 30, 30)` yields buffer cells with that background.
- `miller_border_bg` and `split_border_bg` differ in rendered output when configured.
