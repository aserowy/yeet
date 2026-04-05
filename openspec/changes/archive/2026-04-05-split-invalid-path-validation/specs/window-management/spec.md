## ADDED Requirements

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
