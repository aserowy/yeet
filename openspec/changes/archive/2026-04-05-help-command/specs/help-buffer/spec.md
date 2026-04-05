## ADDED Requirements

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
The help buffer SHALL display markdown content with syntax highlighting using the existing syntect-based highlighting infrastructure (`yeet-frontend/src/task/syntax.rs`).

#### Scenario: Help page rendered with markdown highlighting
- **WHEN** a help page is opened
- **THEN** the content SHALL be syntax highlighted as markdown using syntect

### Requirement: Help buffer supports navigation mode
The help buffer SHALL support navigation mode keybindings for scrolling (j/k, Ctrl+d/Ctrl+u, gg, G) so the user can read through help content.

#### Scenario: User scrolls help buffer with j/k
- **WHEN** the help buffer is focused and the user presses `j` or `k`
- **THEN** the viewport scrolls down or up by one line respectively

#### Scenario: User jumps to top/bottom of help buffer
- **WHEN** the help buffer is focused and the user presses `gg` or `G`
- **THEN** the viewport jumps to the top or bottom of the help content respectively
