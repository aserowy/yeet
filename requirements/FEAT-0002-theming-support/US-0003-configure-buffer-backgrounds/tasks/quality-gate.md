# Quality Gate: Configure Buffer and Border Backgrounds

## Story Summary

Users can configure background colors for buffer surfaces, miller column borders in the directory window, and split borders, with defaults that preserve the current appearance.

## Scope Integrity Checks

- Theme palette includes explicit fields for buffer backgrounds, miller column border backgrounds, and split border backgrounds.
- Defaults mirror today’s rendering (no visual changes when configuration is unset).
- Rendering uses palette values for all three background surfaces.

## Dependencies & Ordering

1. Add palette fields + defaults + Lua override mapping.
2. Apply palette values in buffer/border rendering.
3. Add tests validating palette-driven backgrounds.

## Acceptance Criteria Coverage

- Buffer surfaces render with the configured background color.
- Miller column borders render with the configured border background color.
- Split borders render with the configured border background color.
- Defaults preserve the current look.

## Non-Goals

- Foreground color theming for borders or text (unless needed to preserve current visuals).
- New UI surfaces beyond buffers and borders.
- Runtime theme switching while the app is running.

## Test Expectations

- Tests cover buffer background rendering for at least one buffer type.
- Tests cover split border and miller column border backgrounds.
- Tests confirm defaults do not change existing output.

## Risks & Mitigations

- **Risk**: Split borders and miller borders share rendering code, causing mixed styling.
  - **Mitigation**: Thread explicit border background values per viewport context.
