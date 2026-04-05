## MODIFIED Requirements

### Requirement: Close focused leaf in single split
When the root window is a split and the focused child is a leaf, closing SHALL replace the split with the unfocused child and drop the focused leaf.

#### Scenario: Close focused first child in horizontal split
- **WHEN** the root window is a Horizontal split with focus on the first child (a leaf)
- **THEN** the root window SHALL be replaced by the second child
- **AND** the first child's buffers SHALL be cleaned up

#### Scenario: Close focused second child in vertical split
- **WHEN** the root window is a Vertical split with focus on the second child (a leaf)
- **THEN** the root window SHALL be replaced by the first child
- **AND** the second child's buffers SHALL be cleaned up

## ADDED Requirements

### Requirement: Command error paths transition mode back
When a command executed from the commandline fails with an error, the mode SHALL always transition from Command mode back to the previous mode (Normal or Navigation), regardless of the error path taken within the command handler.

#### Scenario: Split command fails due to invalid path argument
- **WHEN** the user executes `:split <invalid-path>` and path expansion fails
- **THEN** the error message SHALL be displayed on the commandline in red
- **AND** the mode SHALL transition from Command to the previous mode (Normal or Navigation)
- **AND** pressing `:` SHALL clear the error and open a fresh command prompt

#### Scenario: Split command fails due to missing preview path
- **WHEN** the user executes `:split` and no current path can be resolved
- **THEN** the error message SHALL be displayed on the commandline in red
- **AND** the mode SHALL transition from Command to the previous mode

#### Scenario: Vsplit command fails due to invalid path argument
- **WHEN** the user executes `:vsplit <invalid-path>` and path expansion fails
- **THEN** the error message SHALL be displayed on the commandline in red
- **AND** the mode SHALL transition from Command to the previous mode

#### Scenario: Vsplit command fails due to missing preview path
- **WHEN** the user executes `:vsplit` and no current path can be resolved
- **THEN** the error message SHALL be displayed on the commandline in red
- **AND** the mode SHALL transition from Command to the previous mode
