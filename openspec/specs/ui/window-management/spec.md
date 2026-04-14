## Purpose

Specification for closing the currently focused window pane, handling single splits, nested splits, root leaves, and buffer cleanup.

## Requirements

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

### Requirement: Close focused leaf in nested split
When the focused leaf is inside a nested split (a split within a split), closing SHALL only remove the innermost split containing the focused leaf, preserving all other splits and windows in the tree.

#### Scenario: Close leaf in doubly-nested split
- **WHEN** the window tree is Root(Horizontal) → first: Vertical(A, B focused) → second: C, and focus path leads to B
- **THEN** closing SHALL replace the inner Vertical split with A (the sibling of B)
- **AND** the root Horizontal split SHALL remain with first: A, second: C
- **AND** only B's buffers SHALL be cleaned up

#### Scenario: Close leaf in triply-nested split
- **WHEN** the window tree has three levels of nesting and the focused leaf is at the deepest level
- **THEN** closing SHALL only collapse the deepest split containing the focused leaf
- **AND** all other splits and windows SHALL remain unchanged

### Requirement: Close when root is a leaf
When the root window is a leaf (no splits), closing SHALL emit a quit message instead of modifying the window tree.

#### Scenario: Quit when no splits exist
- **WHEN** the root window is a Directory, QuickFix, or Tasks leaf
- **THEN** the system SHALL emit a Quit message with the specified QuitMode
- **AND** the window tree SHALL remain unchanged

### Requirement: Discard changes on close
When `discard_changes` is true, closing SHALL reset unsaved changes for all buffer IDs in the dropped subtree.

#### Scenario: Discard changes for dropped leaf buffers
- **WHEN** closing a focused leaf with `discard_changes` set to true
- **THEN** all buffers belonging to the dropped leaf SHALL have their unsaved changes reset

#### Scenario: Preserve changes when discard is false
- **WHEN** closing a focused leaf with `discard_changes` set to false
- **THEN** no buffer changes SHALL be reset

### Requirement: Split target path must exist
When executing a split or vsplit command with a path argument, the system SHALL validate that the resolved target path exists on disk before creating the split window. If the path does not exist, the system SHALL display an error and not create the split.

#### Scenario: Split with non-existent relative path
- **WHEN** the user executes `:split <relative-path>` and the resolved path does not exist on disk
- **THEN** the system SHALL display an error message indicating the path does not exist
- **AND** the system SHALL NOT create a new split window
- **AND** the mode SHALL transition back from Command to the previous mode

#### Scenario: Vsplit with non-existent relative path
- **WHEN** the user executes `:vsplit <relative-path>` and the resolved path does not exist on disk
- **THEN** the system SHALL display an error message indicating the path does not exist
- **AND** the system SHALL NOT create a new split window
- **AND** the mode SHALL transition back from Command to the previous mode

#### Scenario: Split with valid existing path
- **WHEN** the user executes `:split <path>` and the resolved path exists on disk
- **THEN** the system SHALL create the split window and navigate to the target path

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
