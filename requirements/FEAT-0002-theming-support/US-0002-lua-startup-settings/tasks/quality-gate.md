# Quality Gate: Define Settings via Lua on Startup

## Story Summary

On startup, the application discovers and executes a Lua configuration file using the documented config path strategy, and the Lua-defined theme settings are applied to the UI theme palette via the mlua engine.

## Scope Integrity Checks

- Lua configuration discovery uses a single, documented search path strategy: `$XDG_CONFIG_HOME/yeet/init.lua` with fallback to `~/.config/yeet/init.lua`.
- Startup applies Lua-defined settings before rendering begins.
- Theme settings in Lua map only to the existing `ThemePalette` fields.
- Lua execution uses the `mlua` engine.

## Dependencies & Ordering

1. Discover and load Lua config on startup.
2. Define the Lua API for theme palette settings.
3. Add tests for Lua discovery and theme application.

## Acceptance Criteria Coverage

- Startup executes a discovered Lua config.
- Theme settings defined in Lua are applied to the palette used by rendering.

## Non-Goals

- General-purpose Lua scripting beyond theme settings.
- CLI flags for selecting Lua config files.
- Dynamic reload of Lua settings while running.

## Test Expectations

- Tests validate discovery behavior and a no-config path.
- Tests validate Lua-defined palette overrides vs defaults.

## Risks & Mitigations

- **Risk**: Ambiguous config discovery leads to unexpected files executing.
  - **Mitigation**: Document and test a single discovery strategy with deterministic priority.
- **Risk**: Lua errors block startup without visibility.
  - **Mitigation**: Surface errors via existing logging/print channels with clear context.
