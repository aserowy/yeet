## MODIFIED Requirements

### Requirement: Help pages are bundled markdown files
The system SHALL embed help pages as markdown files at compile time. Help pages SHALL be available without any external file dependencies at runtime. The help system SHALL include pages for: index, commands, keybindings, modes, and configuration.

#### Scenario: Help content available in release binary
- **WHEN** the application is built as a release binary
- **THEN** all help pages (index, commands, keybindings, modes, configuration) are embedded in the binary and accessible via the `:help` command

#### Scenario: New modes page is accessible
- **WHEN** the user executes `:help modes`
- **THEN** the modes help page is displayed covering all four modes and their transition semantics

#### Scenario: New configuration page is accessible
- **WHEN** the user executes `:help configuration`
- **THEN** the configuration help page is displayed covering Lua config, theme tokens, and syntect theme selection

### Requirement: Help pages follow a consistent entry structure
Help pages SHALL use a structured markdown format with three levels: page title (`#`), section (`##`), and entry (`### \`identifier\``). Each entry SHALL use a level-3 heading with the entry identifier wrapped in backticks. Each entry description SHALL be at minimum two sentences long, providing both what the feature does and relevant context or constraints.

#### Scenario: Help page has structured entries
- **WHEN** a help page contains command documentation
- **THEN** each command SHALL be a level-3 heading with the command name in backticks (e.g., `### \`help\``)

#### Scenario: Entry identifiers are unique within a page
- **WHEN** a help page contains multiple entries
- **THEN** each entry identifier SHALL be unique within that page

#### Scenario: Entry descriptions are detailed
- **WHEN** a help entry is displayed
- **THEN** the description SHALL be at minimum two sentences, covering what the feature does and any relevant constraints, related features, or behavior nuances

### Requirement: Help pages cover all implemented functionality
Every implemented command, keybinding, mode, and configurable option SHALL be documented in at least one help page. No functionality SHALL exist only in README.md without a corresponding help entry.

#### Scenario: All keybindings are documented
- **WHEN** a user searches for any implemented keybinding via `:help`
- **THEN** the keybinding SHALL be found in the keybindings help page

#### Scenario: All commands are documented
- **WHEN** a user searches for any implemented command via `:help`
- **THEN** the command SHALL be found in the commands help page
