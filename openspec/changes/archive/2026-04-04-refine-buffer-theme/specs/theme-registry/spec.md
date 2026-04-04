## MODIFIED Requirements

### Requirement: Color token names cover all UI elements
The system SHALL define color tokens for at minimum the following UI elements:
- Tabbar: active tab foreground/background, inactive tab foreground/background, tabbar background
- Statusline: foreground, background (focused and unfocused variants), diff added/modified/removed colors, permissions foreground, border foreground, border background
- Buffer: cursor line background, search highlight background, line number foreground, current line number foreground, sign column foreground, file entry foreground, directory entry foreground
- Directory window: border foreground, border background
- Split: border foreground, border background
- Signs: quickfix sign color, mark sign color

The system SHALL NOT define token constants for UI elements that have no rendering code consuming them. Specifically, `CommandLineFg`, `CommandLineBg`, `CursorNormal`, `CursorInsert`, and `syntax` (as a token constant) SHALL NOT exist as constants or default registrations until rendering code uses them.

#### Scenario: All hardcoded colors have corresponding tokens
- **WHEN** the default theme is loaded with no user overrides
- **THEN** the rendered UI is visually identical to the current hardcoded appearance

#### Scenario: No dead token constants
- **WHEN** inspecting the `tokens` module
- **THEN** every constant is referenced by at least one rendering or theme conversion function outside of the `Default` impl

## REMOVED Requirements

### Requirement: Commandline and cursor mode token constants
**Reason**: `COMMANDLINE_FG`, `COMMANDLINE_BG`, `CURSOR_NORMAL`, `CURSOR_INSERT`, and `SYNTAX_THEME` are dead code — defined as constants but never consumed by any view or rendering function. `COMMANDLINE_FG`/`COMMANDLINE_BG` are registered with default colors that have no effect. `CURSOR_NORMAL`/`CURSOR_INSERT` are not even registered. `SYNTAX_THEME` is never referenced.
**Migration**: Remove the constants and their default registrations. Re-add when rendering code is introduced that consumes them.
