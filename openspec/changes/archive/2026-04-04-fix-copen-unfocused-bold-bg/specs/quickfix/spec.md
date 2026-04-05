## MODIFIED Requirements

### Requirement: Bold rendering of current quickfix entry
The copen buffer SHALL render the entry at `QuickFix.current_index` with ANSI bold styling. All other entries SHALL be rendered without bold. The bold indicator SHALL update whenever `current_index` changes. The buffer background SHALL be preserved through embedded ANSI reset codes in the bold formatting in all focus states — both when the copen window is focused (cursor line background) and when it is unfocused (buffer background).

#### Scenario: Current entry is bold after cfirst
- **WHEN** the copen window is open and the user executes `:cfirst`
- **THEN** the first quickfix entry in the copen buffer SHALL be rendered bold and all other entries SHALL not be bold

#### Scenario: Current entry is bold after cn
- **WHEN** the copen window is open, `current_index` is 0, and the user executes `:cn`
- **THEN** only the next existing entry SHALL be rendered bold in the copen buffer

#### Scenario: Current entry is bold after cN
- **WHEN** the copen window is open and the user executes `:cN`
- **THEN** only the previous existing entry SHALL be rendered bold in the copen buffer

#### Scenario: Cursor line background preserved on current entry
- **WHEN** the copen window is focused and the cursor is on the line matching `QuickFix.current_index`
- **THEN** the entire cursor line SHALL display the cursor line background color, not the buffer background color, despite the ANSI reset in the bold formatting

#### Scenario: Buffer background preserved on unfocused current entry
- **WHEN** the copen window is not focused
- **THEN** the bold-formatted current entry SHALL maintain the buffer background color through the ANSI reset, not reverting to terminal default
