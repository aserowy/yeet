## ADDED Requirements

### Requirement: Locality of behavior for quickfix refresh
Quickfix commands that mutate state SHALL emit the refresh message themselves. The refresh function SHALL handle cross-tab iteration internally. Callers SHALL not need to emit refresh messages after calling qfix mutation functions.

#### Scenario: Command emits refresh
- **WHEN** a qfix command (select_first, next, previous, reset, clear_in, toggle, add) mutates quickfix state
- **THEN** the command SHALL include the refresh emit in its returned actions

#### Scenario: Refresh iterates all tabs
- **WHEN** the refresh function is called
- **THEN** it SHALL iterate all tabs and refresh any quickfix buffer found
