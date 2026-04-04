## MODIFIED Requirements

### Requirement: Buffer line content respects BufferBg after ANSI resets
The system SHALL ensure that every line in the buffer renders with the correct background after ANSI resets. On non-cursor lines, reset sequences SHALL re-apply `BufferBg`. On the cursor line (when cursor line highlighting is active), reset sequences SHALL re-apply `cursor_line_bg` instead of `BufferBg`, so that the cursor line background is continuous across the entire line including after search highlights.

#### Scenario: Non-cursor line has BufferBg background
- **WHEN** `BufferBg` is set to `#1e1e2e` and a line is not the cursor line
- **THEN** the line content renders with `#1e1e2e` background, not the terminal default

#### Scenario: ANSI reset in line number does not clear BufferBg
- **WHEN** `BufferBg` is set to a custom color and a relative line number is rendered with an `\x1b[0m` reset
- **THEN** the background after the reset is `BufferBg`, not the terminal default

#### Scenario: ANSI reset in search highlight does not clear BufferBg on non-cursor line
- **WHEN** `BufferBg` is set to a custom color, a search match highlight ends with a reset, and the line is not the cursor line
- **THEN** the background after the search highlight reset is `BufferBg`, not the terminal default

#### Scenario: ANSI reset in search highlight uses cursor_line_bg on cursor line
- **WHEN** a search match appears on the cursor line and the search highlight ends with a reset
- **THEN** the background after the search highlight reset is `cursor_line_bg`, not `BufferBg`

#### Scenario: Multiple search matches on cursor line preserve cursor_line_bg between matches
- **WHEN** the cursor line contains two or more search matches
- **THEN** the background between and after each match is `cursor_line_bg`

#### Scenario: Search highlight on cursor line with hide_cursor_line uses BufferBg
- **WHEN** a search match appears on a line that would be the cursor line, but `hide_cursor_line` is true
- **THEN** the background after the search highlight reset is `BufferBg`, not `cursor_line_bg`

#### Scenario: ANSI reset in sign does not clear BufferBg
- **WHEN** `BufferBg` is set to a custom color and a sign is rendered with an `\x1b[0m` reset
- **THEN** the background after the sign reset is `BufferBg`, not the terminal default
