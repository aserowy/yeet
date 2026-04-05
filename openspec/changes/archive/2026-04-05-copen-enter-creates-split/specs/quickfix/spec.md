## MODIFIED Requirements

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
