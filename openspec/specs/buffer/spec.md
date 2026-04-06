## Requirements

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

#### Scenario: Wrap is disabled by default
- **WHEN** a ViewPort is created with default settings
- **THEN** `wrap` SHALL be `false` and lines SHALL be horizontally scrolled as before

#### Scenario: Wrap is enabled
- **WHEN** `wrap` is set to `true` on a ViewPort
- **THEN** lines longer than the content width SHALL be broken into multiple visual lines

#### Scenario: Wrap toggled at runtime via set command

- **WHEN** a viewport has wrap disabled and the user executes `:set wrap`
- **THEN** the viewport SHALL re-render with word wrapping enabled
- **THEN** horizontal_index SHALL be reset to 0

#### Scenario: Nowrap toggled at runtime via set command

- **WHEN** a viewport has wrap enabled and the user executes `:set nowrap`
- **THEN** the viewport SHALL re-render without word wrapping

### Requirement: Lines wrap at word boundaries
When wrap is enabled, lines SHALL be broken at the last space character that fits within the viewport content width. If a single word is longer than the content width, the line SHALL be broken at exactly the content width (character-count fallback).

#### Scenario: Line breaks at space boundary
- **WHEN** a line contains "hello world foo" and the content width is 10
- **THEN** the line SHALL be rendered as two visual lines: "hello" and "world foo", breaking at the space before "world"

#### Scenario: Word longer than viewport breaks by character
- **WHEN** a line contains a word with no spaces that is longer than the content width
- **THEN** the word SHALL be broken at exactly the content width boundary

#### Scenario: Line fits within viewport
- **WHEN** a line is shorter than or equal to the content width
- **THEN** the line SHALL be rendered as a single visual line with no wrapping

### Requirement: Continuation lines have no prefix
Continuation lines (all visual lines after the first for a wrapped BufferLine) SHALL NOT display signs, line numbers, or custom prefix. They SHALL be indented with spaces matching the prefix width so their content aligns with the first line's content area.

#### Scenario: First line has line number, continuation does not
- **WHEN** a BufferLine wraps into two visual lines and line numbers are enabled
- **THEN** the first visual line SHALL show the line number and the second SHALL show spaces of the same width

### Requirement: Vertical movement skips wrapped lines
The `j` and `k` motions SHALL move between BufferLine indices, not between visual lines within a wrapped BufferLine. `gg` and `G` SHALL move to the first and last BufferLine respectively.

#### Scenario: j on a wrapped line moves to next BufferLine
- **WHEN** the cursor is on a BufferLine that wraps into 3 visual lines and the user presses `j`
- **THEN** the cursor SHALL move to the next BufferLine, not to the second visual line of the current BufferLine

### Requirement: Horizontal movement traverses across wrap boundaries
All horizontal motions SHALL traverse the full BufferLine content as if it were a single unwrapped line. When the cursor crosses a wrap boundary, it SHALL visually move to the next or previous visual line within the same BufferLine. At the BufferLine boundary, motions SHALL stop (existing boundary behavior preserved). The following motions are affected: `h`, `l`, `w`, `W`, `b`, `B`, `e`, `E`, `ge`, `gE`, `f`, `F`, `t`, `T`, `;`, `,`, `0`, `$`, `I`, `A`, `a`, `x`, `s`, `c<motion>`, `d<motion>`, Left Arrow, Right Arrow, Backspace, and Delete.

#### Scenario: l crosses wrap boundary
- **WHEN** the cursor is at the last character of the first visual line of a wrapped BufferLine and the user presses `l`
- **THEN** the cursor SHALL move to the first character of the second visual line (same BufferLine)

#### Scenario: l at end of BufferLine does nothing
- **WHEN** the cursor is at the last character of the last visual line of a wrapped BufferLine and the user presses `l`
- **THEN** the cursor SHALL not move (existing behavior)

#### Scenario: w crosses wrap boundary
- **WHEN** the next word starts on the second visual line of the same BufferLine and the user presses `w`
- **THEN** the cursor SHALL move to the start of that word on the second visual line

#### Scenario: f finds character across wrap boundary
- **WHEN** the target character of `f<char>` is on the second visual line of the same BufferLine
- **THEN** the cursor SHALL move to that character on the second visual line

#### Scenario: 0 moves to start of BufferLine from continuation line
- **WHEN** the cursor is on the second visual line of a wrapped BufferLine and the user presses `0`
- **THEN** the cursor SHALL move to the first character of the first visual line (BufferLine start)

#### Scenario: $ moves to end of BufferLine from first visual line
- **WHEN** the cursor is on the first visual line of a wrapped BufferLine and the user presses `$`
- **THEN** the cursor SHALL move to the last character of the last visual line (BufferLine end)

### Requirement: Cursor line background spans all wrapped lines
When the cursor is on a wrapped BufferLine, the cursor line background color SHALL be applied to all visual lines of that BufferLine, not just the visual line containing the cursor character.

#### Scenario: Cursor on wrapped line highlights all segments
- **WHEN** a BufferLine wraps into 3 visual lines and the cursor is on that BufferLine
- **THEN** all 3 visual lines SHALL have the cursor line background color

### Requirement: Horizontal scrolling is disabled when wrap is enabled
When `wrap` is `true`, `viewport.horizontal_index` SHALL be forced to `0`. The viewport SHALL NOT horizontally scroll wrapped content.

#### Scenario: Horizontal index reset with wrap
- **WHEN** `wrap` is `true` and the viewport would normally need horizontal scrolling
- **THEN** `horizontal_index` SHALL remain `0` and the content SHALL wrap instead

### Requirement: Viewport accounts for wrapped line heights
When wrap is enabled, the viewport SHALL calculate the visual height of each BufferLine (number of visual lines after wrapping) to determine how many BufferLines fit on screen. Vertical scrolling SHALL ensure the cursor's BufferLine is fully visible, including all its wrapped visual lines.

#### Scenario: Tall wrapped line reduces visible BufferLines
- **WHEN** a BufferLine wraps into 5 visual lines and the viewport height is 10
- **THEN** fewer total BufferLines SHALL fit on screen compared to when the same line is not wrapped

#### Scenario: Scroll ensures cursor line is fully visible
- **WHEN** the cursor is on a BufferLine that wraps into 4 visual lines and only 2 of those lines are within the viewport
- **THEN** the viewport SHALL scroll to show all 4 visual lines of the cursor's BufferLine

### Requirement: Wrapped continuation lines preserve ANSI styling
When a styled line wraps into multiple visual lines, each continuation segment SHALL carry the ANSI escape sequences that were active at the wrap boundary. The visual styling (colors, bold, etc.) SHALL be continuous across all segments of a wrapped line.

#### Scenario: Red-colored line wraps with color preserved
- **WHEN** a line styled with red foreground wraps into two visual lines
- **THEN** both visual lines SHALL render in red foreground

#### Scenario: Multiple style changes within a wrapped line
- **WHEN** a line contains multiple ANSI style changes and wraps at a point after the second style change
- **THEN** the continuation segment SHALL start with the cumulative styling active at the wrap point
