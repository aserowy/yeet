## ADDED Requirements

### Requirement: Close focused leaf in single split
When the root window is a split and the focused child is a leaf, closing SHALL replace the split with the unfocused child and drop the focused leaf.

#### Scenario: Close focused first child in horizontal split
- **WHEN** the root window is a Horizontal split with focus on the first child (a leaf)
- **THEN** the root window SHALL be replaced by the second child
- **AND** the first child's buffers SHALL be cleaned up

#### Scenario: Close focused second child in vertical split
- **WHEN** the root window is a Vertical split with focus on the second child (a leaf)
- **THEN** the root window SHALL be replaced by the first child
- **AND** the second child's buffers SHALL be cleaned up

### Requirement: Close focused leaf in nested split
When the focused leaf is inside a nested split (a split within a split), closing SHALL only remove the innermost split containing the focused leaf, preserving all other splits and windows in the tree.

#### Scenario: Close leaf in doubly-nested split
- **WHEN** the window tree is Root(Horizontal) → first: Vertical(A, B focused) → second: C, and focus path leads to B
- **THEN** closing SHALL replace the inner Vertical split with A (the sibling of B)
- **AND** the root Horizontal split SHALL remain with first: A, second: C
- **AND** only B's buffers SHALL be cleaned up

#### Scenario: Close leaf in triply-nested split
- **WHEN** the window tree has three levels of nesting and the focused leaf is at the deepest level
- **THEN** closing SHALL only collapse the deepest split containing the focused leaf
- **AND** all other splits and windows SHALL remain unchanged

### Requirement: Close when root is a leaf
When the root window is a leaf (no splits), closing SHALL emit a quit message instead of modifying the window tree.

#### Scenario: Quit when no splits exist
- **WHEN** the root window is a Directory, QuickFix, or Tasks leaf
- **THEN** the system SHALL emit a Quit message with the specified QuitMode
- **AND** the window tree SHALL remain unchanged

### Requirement: Discard changes on close
When `discard_changes` is true, closing SHALL reset unsaved changes for all buffer IDs in the dropped subtree.

#### Scenario: Discard changes for dropped leaf buffers
- **WHEN** closing a focused leaf with `discard_changes` set to true
- **THEN** all buffers belonging to the dropped leaf SHALL have their unsaved changes reset

#### Scenario: Preserve changes when discard is false
- **WHEN** closing a focused leaf with `discard_changes` set to false
- **THEN** no buffer changes SHALL be reset
