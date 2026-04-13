## ADDED Requirements

### Requirement: Optional precontent border width override
The `ViewPort` struct SHALL have an optional `precontent_border_width` field (`Option<usize>`) that, when set to `Some(n)`, overrides the computed precontent border width with `n`. When `None` (the default), the existing behavior SHALL be preserved: border width is 1 when `get_precontent_width() > 0`, and 0 otherwise.

#### Scenario: Default is None (computed behavior)
- **WHEN** a `ViewPort` is created with default settings
- **THEN** `precontent_border_width` SHALL be `None` and `get_precontent_border_width()` SHALL return the computed value

#### Scenario: Override to zero suppresses border
- **WHEN** `precontent_border_width` is `Some(0)` and `get_precontent_width() > 0`
- **THEN** `get_precontent_border_width()` SHALL return `0`

#### Scenario: Override enforces width regardless of precontent
- **WHEN** `precontent_border_width` is `Some(2)` and `get_precontent_width() == 0`
- **THEN** `get_precontent_border_width()` SHALL return `2`

### Requirement: Commandline viewport has no precontent border
The commandline buffer's viewport SHALL have `precontent_border_width` set to `Some(0)` to suppress the border space, ensuring the prefix column (used for command count) is rendered without an additional separator cell.

#### Scenario: Commandline viewport border is zero
- **WHEN** a commandline buffer viewport is created with default settings
- **THEN** `precontent_border_width` SHALL be `Some(0)` and `get_precontent_border_width()` SHALL return `0`
