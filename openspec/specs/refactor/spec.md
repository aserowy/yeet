## Purpose

Code quality and structural improvements (TBD).

## Requirements

### Requirement: Combined match patterns for Tasks and QuickFix window variants
Where `Window::Tasks` and `Window::QuickFix` match arms have identical bodies, they SHALL be combined using `|` patterns.

#### Scenario: Duplicate arms are combined
- **WHEN** a match statement has separate `Window::Tasks` and `Window::QuickFix` arms with the same body
- **THEN** they SHALL be combined into a single `Window::QuickFix(vp) | Window::Tasks(vp) =>` arm

### Requirement: Locality of behavior for quickfix refresh
Quickfix commands that mutate state SHALL emit the refresh message themselves. The refresh function SHALL handle cross-tab iteration internally. Callers SHALL not need to emit refresh messages after calling qfix mutation functions.

#### Scenario: Command emits refresh
- **WHEN** a qfix command (select_first, next, previous, reset, clear_in, toggle, add) mutates quickfix state
- **THEN** the command SHALL include the refresh emit in its returned actions

#### Scenario: Refresh iterates all tabs
- **WHEN** the refresh function is called
- **THEN** it SHALL iterate all tabs and refresh any quickfix buffer found

### Requirement: Open handler uses message-based refresh
The quickfix Enter handler in `open.rs` SHALL emit `Message::QuickFixChanged` instead of calling `refresh_quickfix_buffer_in_window` directly, ensuring cross-tab refresh.

#### Scenario: Enter refreshes all tabs
- **WHEN** the user presses Enter on a copen entry
- **THEN** all copen buffers across all tabs SHALL be refreshed
