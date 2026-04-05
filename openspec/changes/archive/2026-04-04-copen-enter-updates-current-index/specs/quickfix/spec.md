## MODIFIED Requirements

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
