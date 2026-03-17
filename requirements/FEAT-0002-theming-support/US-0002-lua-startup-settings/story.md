# User Story: Define Settings via Lua on Startup

## Metadata

- ID: US-0002
- Status: execution
- Feature: FEAT-0002
- As a: user

## Capability

I want: to define theming settings in a Lua config that runs on startup

## Benefit

So that: I can customize the theme using a familiar, scriptable approach without specifying explicit paths

## Acceptance Criteria

- Given a Lua configuration is available through the application's config discovery
- And config discovery checks `$XDG_CONFIG_HOME/yeet/init.lua` with `~/.config/yeet/init.lua` as the fallback
- When the application starts
- Then it executes the Lua config and applies the defined theming settings
