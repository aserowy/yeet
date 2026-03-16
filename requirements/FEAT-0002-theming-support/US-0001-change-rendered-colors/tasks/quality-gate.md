# Quality Gate: Change Rendered Colors

## Story Summary

Users can update the rendered UI colors by setting new values in the theme palette, and those updates are reflected consistently across all rendered UI surfaces.

## Scope Integrity Checks

- Palette fields correspond to every existing hard-coded color in rendering code.
- Default palette values preserve current visual output.
- Rendering code consumes palette values instead of constants.

## Dependencies & Ordering

1. Define palette structure + defaults in Settings.
2. Apply palette to view rendering.
3. Document palette usage.

## Acceptance Criteria Coverage

- Setting new palette values yields consistent UI color updates.
- No regression in default UI appearance when palette is not customized.

## Non-Goals

- User-facing configuration / CLI / file parsing.
- Layout or typography theming.
- New UI surfaces beyond existing rendered elements.

## Test Expectations

- Existing tests pass with palette defaults.
- Any affected tests are updated to reflect palette-driven styles.

## Risks & Mitigations

- **Risk**: Missed hard-coded colors result in inconsistent theming.
  - **Mitigation**: Review all `Color::` usage in view rendering and ensure palette coverage.
