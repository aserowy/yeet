## Purpose

The `:set` command allows users to modify editor options at runtime.

## Requirements

### Requirement: Set wrap enables word wrapping

The system SHALL enable word wrapping on the current window when the user executes the `:set wrap` command.

For Directory windows, the system SHALL enable word wrapping on all three viewports (parent, current, preview).

For single-viewport windows (QuickFix, Tasks, Help), the system SHALL enable word wrapping on that viewport.

For split windows, the system SHALL enable word wrapping on the focused leaf window.

#### Scenario: Enable wrap on a Directory window

- **WHEN** the user executes `:set wrap` while a Directory window is focused
- **THEN** all three viewports (parent, current, preview) SHALL have word wrapping enabled
- **THEN** the buffer content SHALL re-render with wrapped lines

#### Scenario: Enable wrap on a single-viewport window

- **WHEN** the user executes `:set wrap` while a Help, Tasks, or QuickFix window is focused
- **THEN** that window's viewport SHALL have word wrapping enabled

#### Scenario: Enable wrap when already enabled

- **WHEN** the user executes `:set wrap` while word wrapping is already enabled
- **THEN** the system SHALL remain in the wrap-enabled state with no error

### Requirement: Set nowrap disables word wrapping

The system SHALL disable word wrapping on the current window when the user executes the `:set nowrap` command.

For Directory windows, the system SHALL disable word wrapping on all three viewports (parent, current, preview).

For single-viewport windows (QuickFix, Tasks, Help), the system SHALL disable word wrapping on that viewport.

For split windows, the system SHALL disable word wrapping on the focused leaf window.

#### Scenario: Disable wrap on a Directory window

- **WHEN** the user executes `:set nowrap` while a Directory window is focused
- **THEN** all three viewports (parent, current, preview) SHALL have word wrapping disabled
- **THEN** the buffer content SHALL re-render without wrapped lines

#### Scenario: Disable wrap on a single-viewport window

- **WHEN** the user executes `:set nowrap` while a Help, Tasks, or QuickFix window is focused
- **THEN** that window's viewport SHALL have word wrapping disabled

#### Scenario: Disable wrap when already disabled

- **WHEN** the user executes `:set nowrap` while word wrapping is already disabled
- **THEN** the system SHALL remain in the nowrap state with no error

### Requirement: Invalid set arguments produce an error

The system SHALL display an error message when the user executes `:set` with an unrecognized argument.

#### Scenario: Unknown set option

- **WHEN** the user executes `:set foobar`
- **THEN** the system SHALL display an error message indicating the option is unknown

#### Scenario: Empty set argument

- **WHEN** the user executes `:set` with no arguments
- **THEN** the system SHALL display an error message indicating a missing argument
