## MODIFIED Requirements

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
