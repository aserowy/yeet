## MODIFIED Requirements

### Requirement: Embedded help pages are bundled at compile time
The system SHALL embed help pages as markdown files at compile time. Help pages SHALL be available without any external file dependencies at runtime. The help system SHALL include pages for: index, commands, keybindings, modes, configuration, theme, and hooks. Core help pages SHALL NOT contain plugin-specific documentation such as icon resolution guidance, plugin-specific token references, or plugin implementation patterns.

#### Scenario: Help content available in release binary
- **WHEN** the application is built as a release binary
- **THEN** all help pages (index, commands, keybindings, modes, configuration, theme, hooks) are embedded in the binary and accessible via the `:help` command

#### Scenario: Core hooks help page documents only core API
- **WHEN** the user executes `:help hooks`
- **THEN** the hooks help page documents the `y.hook` table, hook registration, context fields, and buffer metadata — without plugin-specific guidance like trailing slash conventions or icon resolution patterns

#### Scenario: New modes page is accessible
- **WHEN** the user executes `:help modes`
- **THEN** the modes help page is displayed covering all four modes and their transition semantics

#### Scenario: New configuration page is accessible
- **WHEN** the user executes `:help configuration`
- **THEN** the configuration help page is displayed as an index covering config file location, error handling, and links to theme and hooks pages

#### Scenario: Theme page is accessible
- **WHEN** the user executes `:help theme`
- **THEN** the theme help page is displayed covering the `y.theme` table, syntax theme selection, and all color token references

#### Scenario: Hooks page is accessible
- **WHEN** the user executes `:help hooks`
- **THEN** the hooks help page is displayed covering the `y.hook` table and all registered hook callbacks — without plugin-specific content

### Requirement: Help pages are maintained alongside functionality
When functionality is added or changed, the corresponding help markdown files SHALL be updated or extended to reflect the new capabilities. Plugin-specific documentation changes SHALL be made in the plugin's own `docs/help/` directory.

#### Scenario: New command added
- **WHEN** a new command is added to the application
- **THEN** the help pages SHALL be updated to document the new command

#### Scenario: Existing functionality changed
- **WHEN** an existing command or keybinding is modified
- **THEN** the relevant help page SHALL be updated to reflect the change

#### Scenario: Plugin-specific docs updated in plugin repo
- **WHEN** plugin behavior changes
- **THEN** the plugin's own `docs/help/` files SHALL be updated, not core help files
