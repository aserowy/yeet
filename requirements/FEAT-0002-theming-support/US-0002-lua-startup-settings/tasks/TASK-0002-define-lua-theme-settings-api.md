# Task: Define Lua API for Theme Settings

## Metadata

- ID: TASK-0002
- Status: execution
- Userstory: US-0002

## Motivation

Users need a clear Lua API to set theme palette values so their configuration can reliably update UI colors on startup.

## Relevant Acceptance Criteria

- Given a Lua configuration is available through the application's config discovery
- When the application starts
- Then it executes the Lua config and applies the defined theming settings

## Requirements

- Define a minimal Lua API surface dedicated to theme palette settings (no general scripting) under a top-level `y` table.
- Map Lua-defined values from `y.theme` directly to existing `ThemePalette` fields, keeping field names consistent or clearly documented.
- Support overriding a subset of palette values while leaving unspecified values at defaults.
- Validate value types and report errors for invalid Lua values (e.g., non-color values) without crashing.
- Apply parsed Lua settings from `y.theme` to the `Settings.theme` before rendering begins.

## Exclusions

- Do NOT add new palette fields beyond the existing `ThemePalette` model.
- Do NOT execute arbitrary Lua actions beyond reading theme configuration.
- Do NOT implement config discovery or file IO in this task.

## Context

- @yeet-frontend/src/settings.rs - `ThemePalette` definition and defaults.
- @yeet-frontend/src/lib.rs - startup flow where settings are applied.
- @AGENTS.md - build/test/lint commands.

## Implementation Plan

### Step 1: Define Lua schema and mapping

Use a top-level `y` table and map `y.theme` keys to `ThemePalette` fields. Document the mapping in code comments for maintainability.

### Step 2: Implement parsing and validation

Add a parser that converts Lua values into `Color` values. If a value is invalid, log a clear error and keep the default for that field.

### Step 3: Apply overrides to settings

Merge parsed values with `ThemePalette::default()` and assign to `Settings.theme` before any rendering.

## Examples

```lua
y = {
  theme = {
    tab_active_bg = "#87CEFA",
    statusline_fg = "#FFFFFF",
  }
}
```

- Only `tab_active_bg` and `statusline_fg` override defaults; all other palette values remain unchanged.
