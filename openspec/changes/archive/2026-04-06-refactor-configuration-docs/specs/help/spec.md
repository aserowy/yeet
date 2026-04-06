## MODIFIED Requirements

### Requirement: Embedded help pages are bundled at compile time

The system SHALL embed help pages as markdown files at compile time. Help pages SHALL be available without any external file dependencies at runtime. The help system SHALL include pages for: index, commands, keybindings, modes, configuration, theme, and hooks.

#### Scenario: Help content available in release binary

- **WHEN** the application is built as a release binary
- **THEN** all help pages (index, commands, keybindings, modes, configuration, theme, hooks) are embedded in the binary and accessible via the `:help` command

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
- **THEN** the hooks help page is displayed covering the `y.hook` table and all registered hook callbacks
