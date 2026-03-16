# Task: Document Palette Usage

## Metadata

- ID: TASK-0003
- Status: done
- Userstory: US-0001

## Motivation

Developers need a clear reference for what palette fields exist and where they are used, ensuring consistent future changes and preventing regressions.

## Relevant Acceptance Criteria

- Given the application supports theming
- When I set new color values for the theme palette
- Then the rendered UI uses the updated colors consistently

## Requirements

- Document the theme palette fields and the UI surfaces they map to.
- Capture any assumptions about defaults and expected behavior.
- Keep documentation close to the palette definition (e.g., in code comments or a README in the story scope, if present).

## Exclusions

- Do NOT add user-facing documentation or configuration instructions.
- Do NOT change rendering logic.

## Context

- @yeet-frontend/src/settings.rs — palette definition location.
- @yeet-frontend/src/view/statusline.rs — example consumers.
- @yeet-frontend/src/view/tabbar.rs — example consumers.

## Implementation Plan

### Step 1: Add palette field documentation

Use rustdoc comments to describe what each palette field controls.

### Step 2: Add usage notes

Document that defaults mirror current colors and that all view colors are sourced from the palette.

## Examples

```rust
/// Background color for the active tab label.
pub tab_active_bg: Color,
```
