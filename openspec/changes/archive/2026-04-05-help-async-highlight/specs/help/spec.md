## MODIFIED Requirements

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
