# Task: Add Tests for Lua Startup Settings

## Metadata

- ID: TASK-0003
- Status: done
- Userstory: US-0002

## Motivation

Tests ensure the Lua discovery and theme application behavior are stable and regressions are caught when configuration or startup code changes.

## Relevant Acceptance Criteria

- Given a Lua configuration is available through the application's config discovery
- When the application starts
- Then it executes the Lua config and applies the defined theming settings

## Requirements

- Add tests for the Lua config discovery helper, including both found and not-found paths.
- Add tests that validate Lua-defined palette overrides from `y.theme` apply correctly and defaults remain for unspecified fields.
- Add tests that invalid Lua values surface errors and fall back to defaults.
- Ensure tests are deterministic and do not depend on the user’s real filesystem state.

## Exclusions

- Do NOT add integration tests that require launching the full TUI.
- Do NOT expand test scope to unrelated settings or CLI flags.

## Context

- @yeet/src/main.rs - startup settings construction, potential discovery helper location.
- @yeet-frontend/src/settings.rs - `ThemePalette` defaults for comparisons.
- @AGENTS.md - build/test/lint commands.

## Implementation Plan

### Step 1: Test discovery helper with controlled paths

Provide a test harness that sets up a temporary config directory and asserts the helper returns the expected path when `init.lua` exists, and `None` when it does not.

### Step 2: Test Lua palette overrides

Use an in-memory Lua snippet (or temp file) to set a subset of palette values under `y.theme` and assert the resulting `ThemePalette` has overrides + defaults.

### Step 3: Test invalid Lua values

Provide a Lua config with invalid values and assert errors are surfaced and defaults are used.

## Examples

- With Lua `y = { theme = { statusline_fg = "#FFFFFF" } }`, the resulting palette matches defaults except for `statusline_fg`.
- With Lua `y = { theme = { statusline_fg = 12 } }`, log an error and keep `statusline_fg` at default.
