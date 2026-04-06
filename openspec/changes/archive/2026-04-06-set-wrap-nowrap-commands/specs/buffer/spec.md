## MODIFIED Requirements

### Requirement: Line wrapping

Buffers SHALL support a word wrapping mode that breaks long lines at word boundaries to fit within the viewport width.

When word wrapping is enabled on a viewport, the view layer SHALL:

- Break lines at space characters when possible, falling back to character boundaries
- Preserve ANSI escape sequences across wrapped segments
- Display line numbers and signs only on the first visual line of a wrapped logical line
- Indent continuation lines to align with the content column of the first line
- Adjust cursor positioning to account for wrapped segments
- Adjust vertical scrolling to account for visual line heights

When word wrapping is disabled, horizontal scrolling SHALL be used for lines exceeding viewport width.

Word wrapping MAY be toggled at runtime via the `:set wrap` and `:set nowrap` commands.

#### Scenario: Wrap toggled at runtime via set command

- **WHEN** a viewport has wrap disabled and the user executes `:set wrap`
- **THEN** the viewport SHALL re-render with word wrapping enabled
- **THEN** horizontal_index SHALL be reset to 0

#### Scenario: Nowrap toggled at runtime via set command

- **WHEN** a viewport has wrap enabled and the user executes `:set nowrap`
- **THEN** the viewport SHALL re-render without word wrapping
