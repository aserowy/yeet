## ADDED Requirements

### Requirement: Help pages are bundled markdown files
The system SHALL embed help pages as markdown files at compile time. Help pages SHALL be available without any external file dependencies at runtime.

#### Scenario: Help content available in release binary
- **WHEN** the application is built as a release binary
- **THEN** all help pages are embedded in the binary and accessible via the `:help` command

### Requirement: Help pages follow a consistent entry structure
Help pages SHALL use a structured markdown format with three levels: page title (`#`), section (`##`), and entry (`### \`identifier\``). Each entry SHALL use a level-3 heading with the entry identifier wrapped in backticks. This structure enables programmatic navigation (e.g., jump to next/previous entry).

#### Scenario: Help page has structured entries
- **WHEN** a help page contains command documentation
- **THEN** each command SHALL be a level-3 heading with the command name in backticks (e.g., `### \`help\``)

#### Scenario: Entry identifiers are unique within a page
- **WHEN** a help page contains multiple entries
- **THEN** each entry identifier SHALL be unique within that page

#### Scenario: Topic resolution matches all structural levels
- **WHEN** the user executes `:help <topic>`
- **THEN** the system SHALL match `<topic>` against page titles (`#`), section headings (`##`), and entry identifiers (`` ### `identifier` ``) across all help pages and scroll to the matching location

### Requirement: Help pages are maintained alongside functionality
When functionality is added or changed, the corresponding help markdown files SHALL be updated or extended to reflect the new capabilities.

#### Scenario: New command added
- **WHEN** a new command is added to the application
- **THEN** the help pages SHALL be updated to document the new command

#### Scenario: Existing functionality changed
- **WHEN** an existing command or keybinding is modified
- **THEN** the relevant help page SHALL be updated to reflect the change
