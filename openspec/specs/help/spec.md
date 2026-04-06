## Requirements

### Requirement: Help buffer is read-only
The help buffer SHALL NOT allow modification of its content. The buffer SHALL be displayed in read-only mode.

#### Scenario: Help buffer rejects edits
- **WHEN** the help buffer is displayed
- **THEN** the user SHALL NOT be able to modify the buffer content

### Requirement: Help buffer can be closed
The user SHALL be able to close the help buffer using the standard window close command (`:q`). Closing the help buffer SHALL remove the split and return focus to the remaining window.

#### Scenario: User closes help buffer with :q
- **WHEN** the user focuses the help pane and executes `:q`
- **THEN** the help pane is closed, the split is removed, and focus returns to the remaining window

### Requirement: Help buffer content is syntax highlighted
The help buffer SHALL display markdown content with syntax highlighting using the existing syntect-based highlighting infrastructure. Highlighting SHALL be performed asynchronously via the task system, following the same pattern as file preview highlighting. The help buffer SHALL be displayed immediately with unhighlighted content, then updated in-place when highlighting completes.

#### Scenario: Help page rendered with markdown highlighting
- **WHEN** a help page is opened
- **THEN** the content SHALL be syntax highlighted as markdown using syntect via an asynchronous task

#### Scenario: Help buffer is immediately readable before highlighting completes
- **WHEN** the user executes `:help` and the highlighting task has not yet completed
- **THEN** the help buffer SHALL display the raw markdown content and be fully navigable

#### Scenario: Highlighting updates buffer in-place
- **WHEN** the highlighting task completes and the help buffer is still open
- **THEN** the help buffer content SHALL be replaced with highlighted lines without changing the cursor or viewport position

#### Scenario: Multiple help buffers highlighted independently
- **WHEN** multiple help buffers are open simultaneously (e.g., in different splits or tabs)
- **THEN** each help buffer SHALL receive its own highlighting task and be updated independently by buffer_id

#### Scenario: Help buffer closed before highlighting completes
- **WHEN** the user closes the help buffer before the highlighting task delivers its result
- **THEN** the highlighting result SHALL be silently discarded with no error

### Requirement: Help buffer supports navigation mode
The help buffer SHALL support navigation mode keybindings for scrolling (j/k, Ctrl+d/Ctrl+u, gg, G) so the user can read through help content.

#### Scenario: User scrolls help buffer with j/k
- **WHEN** the help buffer is focused and the user presses `j` or `k`
- **THEN** the viewport scrolls down or up by one line respectively

#### Scenario: User jumps to top/bottom of help buffer
- **WHEN** the help buffer is focused and the user presses `gg` or `G`
- **THEN** the viewport jumps to the top or bottom of the help content respectively

### Requirement: Open help index with bare help command
The system SHALL open a help index page when the user executes the `:help` command without arguments. The help index SHALL be displayed as a read-only buffer in a horizontal split below the current window.

#### Scenario: User runs :help with no arguments
- **WHEN** the user executes `:help`
- **THEN** a horizontal split is created with the help index page displayed in the bottom pane as a read-only buffer

#### Scenario: Help split receives focus
- **WHEN** the help buffer opens in a horizontal split
- **THEN** the help pane SHALL receive focus

### Requirement: Open help for a specific topic
The system SHALL resolve `<topic>` against all three structural levels of help pages: page titles (`#`), section headings (`##`), and entry identifiers (`` ### `identifier` ``). Topic matching SHALL be case-insensitive. The matching help page SHALL be displayed as a read-only buffer in a horizontal split below the current window, scrolled so the matching heading is at the top of the visible viewport area.

#### Scenario: Topic matches a page title
- **WHEN** the user executes `:help <topic>` where `<topic>` matches a page title (`#` heading)
- **THEN** the matching help page is opened at the beginning

#### Scenario: Topic matches a section heading
- **WHEN** the user executes `:help <topic>` where `<topic>` matches a section heading (`##`) within a help page
- **THEN** the help page containing that section is opened with the section heading positioned at the top of the viewport

#### Scenario: Topic matches an entry identifier
- **WHEN** the user executes `:help <topic>` where `<topic>` matches an entry identifier (`` ### `identifier` ``)
- **THEN** the help page containing that entry is opened with the entry heading positioned at the top of the viewport

#### Scenario: Topic matches a section heading with different casing
- **WHEN** the user executes `:help file operations` where "File Operations" is a `##` heading
- **THEN** the help page containing that section is opened with the section heading positioned at the top of the viewport

#### Scenario: Topic matches an entry identifier with different casing
- **WHEN** the user executes `:help Split` where `split` is an entry identifier
- **THEN** the help page containing that entry is opened with the entry heading positioned at the top of the viewport

#### Scenario: Topic matches a page name with different casing
- **WHEN** the user executes `:help Commands` where `commands` is a page name
- **THEN** the matching help page is opened at the beginning

#### Scenario: Topic not found
- **WHEN** the user executes `:help <topic>` where `<topic>` does not match any page, section, or entry regardless of casing
- **THEN** the system SHALL display an error message indicating the topic was not found

### Requirement: Help buffer lines produce exactly one rendered line each
Each line in the help buffer SHALL produce exactly one `ratatui::Line` when rendered. Syntax-highlighted content containing embedded newlines SHALL be split into separate `BufferLine`s so that each produces a single rendered line.

#### Scenario: Cursor line width matches viewport width
- **WHEN** the cursor is on a help buffer line that contains syntax-highlighted markdown
- **THEN** the rendered cursor line width SHALL equal the viewport content width, with no visual line break or wrap

#### Scenario: Highlighted string with trailing newline
- **WHEN** a syntax-highlighted string ends with a trailing newline
- **THEN** it SHALL produce exactly one `BufferLine`, not an additional empty line

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

### Requirement: Help pages are maintained alongside functionality
When functionality is added or changed, the corresponding help markdown files SHALL be updated or extended to reflect the new capabilities.

#### Scenario: New command added
- **WHEN** a new command is added to the application
- **THEN** the help pages SHALL be updated to document the new command

#### Scenario: Existing functionality changed
- **WHEN** an existing command or keybinding is modified
- **THEN** the relevant help page SHALL be updated to reflect the change
