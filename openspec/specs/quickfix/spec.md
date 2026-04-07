## Requirements

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
When the user presses `enter` on a selected entry in the copen buffer, the system SHALL navigate to that entry's path in the nearest directory window and SHALL move focus from the copen window to that directory window. The system SHALL also update `QuickFix.current_index` to match the selected entry and refresh the copen buffer so the bold indicator reflects the new current entry. The nearest directory window SHALL be found by: identifying the split that contains the copen buffer, traversing the sibling subtree (the other child of that split), and finding the first `Directory` window by following the focus path. If no sibling directory window exists, the system SHALL create a horizontal split with a new directory window as the first child and the copen window as the second child, focus the directory window, and navigate to the selected path.

#### Scenario: Enter opens path in sibling directory window
- **WHEN** the copen window is the second child of a horizontal split, the first child is a Directory window, and the user presses enter on an entry
- **THEN** the entry's path SHALL be opened in the first child's directory window, focus SHALL move to that directory window, `current_index` SHALL be updated to the selected entry, and the copen buffer SHALL refresh with bold on the new current entry

#### Scenario: Enter opens path in nested sibling directory window
- **WHEN** the copen window is inside a nested split and the sibling subtree contains multiple directory windows
- **THEN** the entry's path SHALL be opened in the directory window found by following the focus path of the sibling subtree, focus SHALL move to that directory window, and `current_index` SHALL be updated

#### Scenario: Enter with no directory window creates split
- **WHEN** the copen window has no sibling directory window and the user presses enter on an entry
- **THEN** a horizontal split SHALL be created with a new directory window as the first child and the copen window as the second child, focus SHALL move to the directory window, the selected path SHALL be navigated to, and `current_index` SHALL be updated

### Requirement: Remove entry with dd
When the user presses `dd` on a selected entry in the copen buffer, the system SHALL remove that entry from the quickfix list, remove its quickfix sign from all directory buffers, rebuild the copen buffer content, and adjust the cursor position. If the cursor index exceeds the number of entries, it SHALL be clamped to the last entry before removal proceeds.

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

#### Scenario: Cursor past end of entries
- **WHEN** the user presses `dd` and the cursor index is greater than or equal to the number of entries
- **THEN** the cursor SHALL be clamped to the last entry and that entry SHALL be removed

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
The copen buffer SHALL be rebuilt whenever the quickfix list is mutated by any command (`:cfirst`, `:cn`, `:cN`, `:clearcl`, toggle, invert, `:cdo`, add). The refresh SHALL find and update the copen buffer regardless of which tab is currently active. The copen buffer SHALL not be removed by buffer cleanup when it exists in an inactive tab.

#### Scenario: Refresh after clearcl
- **WHEN** the copen window is open and the user executes `:clearcl`
- **THEN** the copen buffer SHALL be rebuilt to reflect an empty quickfix list

#### Scenario: Refresh after toggle
- **WHEN** the copen window is open and the user toggles a quickfix entry in a directory window
- **THEN** the copen buffer SHALL be rebuilt to reflect the updated entries

#### Scenario: Refresh from different tab
- **WHEN** the copen window is open in tab A and the user executes `:cn` from tab B
- **THEN** the copen buffer in tab A SHALL be rebuilt to reflect the updated current index

#### Scenario: Copen buffer preserved across tab switches
- **WHEN** the copen window is open in tab A and the user switches to tab B
- **THEN** the copen buffer SHALL not be removed by buffer cleanup and SHALL remain intact when tab A is refocused

#### Scenario: Topen buffer preserved across tab switches
- **WHEN** the topen window is open in tab A and the user switches to tab B
- **THEN** the topen (Tasks) buffer SHALL not be removed by buffer cleanup and SHALL remain intact when tab A is refocused

### Requirement: Copen statusline
The copen window SHALL display a statusline with "QuickFix" as the label (bold when focused) and a position indicator showing cursor position relative to total entries.

#### Scenario: Focused statusline
- **WHEN** the copen window is focused
- **THEN** the statusline SHALL show "QuickFix" in bold and the position as "{cursor+1}/{total}"

#### Scenario: Unfocused statusline
- **WHEN** the copen window is not focused
- **THEN** the statusline SHALL show "QuickFix" without bold styling

### Requirement: Combined match patterns for Tasks and QuickFix window variants
Where `Window::Tasks` and `Window::QuickFix` match arms have identical bodies, they SHALL be combined using `|` patterns.

#### Scenario: Duplicate arms are combined
- **WHEN** a match statement has separate `Window::Tasks` and `Window::QuickFix` arms with the same body
- **THEN** they SHALL be combined into a single `Window::QuickFix(vp) | Window::Tasks(vp) =>` arm

### Requirement: Locality of behavior for quickfix refresh
Quickfix commands that mutate state SHALL emit the refresh message themselves. The refresh function SHALL handle cross-tab iteration internally. Callers SHALL not need to emit refresh messages after calling qfix mutation functions.

#### Scenario: Command emits refresh
- **WHEN** a qfix command (select_first, next, previous, reset, clear_in, toggle, add) mutates quickfix state
- **THEN** the command SHALL include the refresh emit in its returned actions

#### Scenario: Refresh iterates all tabs
- **WHEN** the refresh function is called
- **THEN** it SHALL iterate all tabs and refresh any quickfix buffer found

### Requirement: Open handler uses message-based refresh
The quickfix Enter handler in `open.rs` SHALL emit `Message::QuickFixChanged` instead of calling `refresh_quickfix_buffer_in_window` directly, ensuring cross-tab refresh.

#### Scenario: Enter refreshes all tabs
- **WHEN** the user presses Enter on a copen entry
- **THEN** all copen buffers across all tabs SHALL be refreshed

### Requirement: README documents all implemented commands and keybindings
The README.md SHALL list every implemented command and keybinding in its shortcuts and commands tables.

#### Scenario: copen command is documented
- **WHEN** a user reads the README commands table
- **THEN** the `:copen` command SHALL be listed with its description

#### Scenario: gg and G keybindings are documented
- **WHEN** a user reads the navigation and normal mode keybindings table
- **THEN** `gg` and `G` SHALL be listed for jumping to top/bottom

#### Scenario: Enter keybinding is documented
- **WHEN** a user reads the navigation mode keybindings table
- **THEN** `Enter` SHALL be listed for opening the selected entry
