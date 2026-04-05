## MODIFIED Requirements

### Requirement: Command error paths transition mode back
When a command executed from the commandline fails with an error, the mode SHALL always transition from Command mode back to the previous mode (Normal or Navigation), regardless of the error path taken within the command handler.

#### Scenario: Split command fails due to invalid path argument
- **WHEN** the user executes `:split <invalid-path>` and path expansion fails
- **THEN** the error message SHALL be displayed on the commandline in red
- **AND** the mode SHALL transition from Command to the previous mode (Normal or Navigation)
- **AND** pressing `:` SHALL clear the error and open a fresh command prompt

#### Scenario: Split command fails due to missing preview path
- **WHEN** the user executes `:split` with no arguments and no current path can be resolved and no home directory is available
- **THEN** the error message SHALL be displayed on the commandline in red
- **AND** the mode SHALL transition from Command to the previous mode

#### Scenario: Vsplit command fails due to invalid path argument
- **WHEN** the user executes `:vsplit <invalid-path>` and path expansion fails
- **THEN** the error message SHALL be displayed on the commandline in red
- **AND** the mode SHALL transition from Command to the previous mode

#### Scenario: Vsplit command fails due to missing preview path
- **WHEN** the user executes `:vsplit` with no arguments and no current path can be resolved and no home directory is available
- **THEN** the error message SHALL be displayed on the commandline in red
- **AND** the mode SHALL transition from Command to the previous mode

## ADDED Requirements

### Requirement: Split fallback to home directory for non-directory windows
When the focused window has no associated directory path (e.g., Tasks, QuickFix, or any future non-directory window type) and `:split` or `:vsplit` is executed without arguments, the system SHALL fall back to the user's home directory as the split target.

#### Scenario: Split without arguments from a non-directory window
- **WHEN** the focused window is a non-directory window (no current path available)
- **AND** the user executes `:split` with no arguments
- **THEN** the system SHALL create a horizontal split with a new directory pane targeting the home directory

#### Scenario: Vsplit without arguments from a non-directory window
- **WHEN** the focused window is a non-directory window (no current path available)
- **AND** the user executes `:vsplit` with no arguments
- **THEN** the system SHALL create a vertical split with a new directory pane targeting the home directory

#### Scenario: Split without arguments from a non-directory window when home directory is unavailable
- **WHEN** the focused window is a non-directory window (no current path available)
- **AND** the user executes `:split` with no arguments
- **AND** the home directory cannot be resolved
- **THEN** the system SHALL display an error message
- **AND** the system SHALL NOT create a new split window

### Requirement: Split with absolute path or mark from non-directory windows
When the focused window has no associated directory path and `:split` or `:vsplit` is executed with an absolute path or a mark reference, the system SHALL use that path as the split target.

#### Scenario: Split with absolute path from a non-directory window
- **WHEN** the focused window is a non-directory window (no current path available)
- **AND** the user executes `:split /some/absolute/path`
- **AND** the path exists on disk
- **THEN** the system SHALL create a horizontal split with a new directory pane targeting the absolute path

#### Scenario: Vsplit with mark from a non-directory window
- **WHEN** the focused window is a non-directory window (no current path available)
- **AND** the user executes `:vsplit 'a` where mark `a` is set to an existing path
- **THEN** the system SHALL create a vertical split with a new directory pane targeting the marked path

#### Scenario: Split with absolute path that does not exist from a non-directory window
- **WHEN** the focused window is a non-directory window (no current path available)
- **AND** the user executes `:split /nonexistent/path`
- **THEN** the system SHALL display an error message indicating the path does not exist
- **AND** the system SHALL NOT create a new split window

### Requirement: Split with relative path from non-directory windows SHALL error
When the focused window has no associated directory path and `:split` or `:vsplit` is executed with a relative path, the system SHALL display an error because there is no base directory to resolve the relative path against.

#### Scenario: Split with relative path from a non-directory window
- **WHEN** the focused window is a non-directory window (no current path available)
- **AND** the user executes `:split some/relative/path`
- **THEN** the system SHALL display an error message indicating that relative paths require a directory context
- **AND** the system SHALL NOT create a new split window

#### Scenario: Vsplit with relative path from a non-directory window
- **WHEN** the focused window is a non-directory window (no current path available)
- **AND** the user executes `:vsplit some/relative/path`
- **THEN** the system SHALL display an error message indicating that relative paths require a directory context
- **AND** the system SHALL NOT create a new split window
