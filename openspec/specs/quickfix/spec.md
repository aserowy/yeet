## ADDED Requirements

### Requirement: Open copen window with :copen command
The `:copen` command SHALL create a horizontal split at the currently focused window, placing the copen buffer as the second child with focus set to the copen window. The copen buffer SHALL display all quickfix entries in the same format as `:cl` (index, path, removed status).

#### Scenario: Opening copen with quickfix entries
- **WHEN** the quickfix list contains entries and the user executes `:copen`
- **THEN** a horizontal split is created with the copen buffer as the second child, focus moves to the copen window, and all quickfix entries are displayed with their index and path

#### Scenario: Opening copen with empty quickfix list
- **WHEN** the quickfix list is empty and the user executes `:copen`
- **THEN** a horizontal split is created with an empty copen buffer

#### Scenario: Opening copen when already open
- **WHEN** a copen window already exists in the current tab and the user executes `:copen`
- **THEN** focus SHALL move to the existing copen window without creating a new one

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

### Requirement: Open entry in nearest directory window with enter
When the user presses `enter` on a selected entry in the copen buffer, the system SHALL navigate to that entry's path in the nearest directory window and SHALL move focus from the copen window to that directory window. The system SHALL also update `QuickFix.current_index` to match the selected entry and refresh the copen buffer so the bold indicator reflects the new current entry. The nearest directory window SHALL be found by: identifying the split that contains the copen buffer, traversing the sibling subtree (the other child of that split), and finding the first `Directory` window by following the focus path.

#### Scenario: Enter opens path in sibling directory window
- **WHEN** the copen window is the second child of a horizontal split, the first child is a Directory window, and the user presses enter on an entry
- **THEN** the entry's path SHALL be opened in the first child's directory window, focus SHALL move to that directory window, `current_index` SHALL be updated to the selected entry, and the copen buffer SHALL refresh with bold on the new current entry

#### Scenario: Enter opens path in nested sibling directory window
- **WHEN** the copen window is inside a nested split and the sibling subtree contains multiple directory windows
- **THEN** the entry's path SHALL be opened in the directory window found by following the focus path of the sibling subtree, focus SHALL move to that directory window, and `current_index` SHALL be updated

#### Scenario: Enter with no directory window in sibling
- **WHEN** the sibling subtree of the copen split contains no directory window
- **THEN** pressing enter SHALL have no effect

### Requirement: Remove entry with dd
When the user presses `dd` on a selected entry in the copen buffer, the system SHALL remove that entry from the quickfix list, remove its quickfix sign from all directory buffers, rebuild the copen buffer content, and adjust the cursor position.

#### Scenario: Remove entry before current_index
- **WHEN** the user presses `dd` on an entry whose index is less than `QuickFix.current_index`
- **THEN** the entry SHALL be removed from the quickfix list, `current_index` SHALL be decremented by one, and the copen buffer SHALL be rebuilt

#### Scenario: Remove entry at current_index
- **WHEN** the user presses `dd` on the entry at `QuickFix.current_index`
- **THEN** the entry SHALL be removed from the quickfix list, `current_index` SHALL be clamped to the new entry count, and the copen buffer SHALL be rebuilt

#### Scenario: Remove entry after current_index
- **WHEN** the user presses `dd` on an entry whose index is greater than `QuickFix.current_index`
- **THEN** the entry SHALL be removed from the quickfix list, `current_index` SHALL remain unchanged, and the copen buffer SHALL be rebuilt

#### Scenario: Remove last remaining entry
- **WHEN** the user presses `dd` and only one entry remains in the quickfix list
- **THEN** the entry SHALL be removed, `current_index` SHALL be set to 0, and the copen buffer SHALL show an empty list

#### Scenario: Sign removal on dd
- **WHEN** the user presses `dd` on an entry
- **THEN** the quickfix sign for that entry's path SHALL be removed from all directory buffers

### Requirement: Navigation keymaps match topen
The copen buffer SHALL support the same navigation keymaps as the Tasks (`:topen`) window for cursor movement (j, k, gg, G, and equivalent motions).

#### Scenario: Cursor movement with j and k
- **WHEN** the copen window is focused and the user presses `j` or `k`
- **THEN** the cursor SHALL move down or up respectively within the copen buffer entries

#### Scenario: Jump to top with gg
- **WHEN** the copen window is focused and the user presses `gg`
- **THEN** the cursor SHALL move to the first entry

#### Scenario: Jump to bottom with G
- **WHEN** the copen window is focused and the user presses `G`
- **THEN** the cursor SHALL move to the last entry

### Requirement: Non-mapped keys are no-ops
All keymaps not shared with the topen navigation set and not explicitly mapped (enter, dd) SHALL have no effect in the copen buffer.

#### Scenario: Unmapped key press
- **WHEN** the copen window is focused and the user presses a key that is not a topen navigation key, enter, or dd
- **THEN** nothing SHALL happen

### Requirement: Copen buffer refresh on quickfix mutation
The copen buffer SHALL be rebuilt whenever the quickfix list is mutated by any command (`:cfirst`, `:cn`, `:cN`, `:clearcl`, toggle, invert, `:cdo`, add).

#### Scenario: Refresh after clearcl
- **WHEN** the copen window is open and the user executes `:clearcl`
- **THEN** the copen buffer SHALL be rebuilt to reflect an empty quickfix list

#### Scenario: Refresh after toggle
- **WHEN** the copen window is open and the user toggles a quickfix entry in a directory window
- **THEN** the copen buffer SHALL be rebuilt to reflect the updated entries

### Requirement: Copen statusline
The copen window SHALL display a statusline with "QuickFix" as the label (bold when focused) and a position indicator showing cursor position relative to total entries.

#### Scenario: Focused statusline
- **WHEN** the copen window is focused
- **THEN** the statusline SHALL show "QuickFix" in bold and the position as "{cursor+1}/{total}"

#### Scenario: Unfocused statusline
- **WHEN** the copen window is not focused
- **THEN** the statusline SHALL show "QuickFix" without bold styling
