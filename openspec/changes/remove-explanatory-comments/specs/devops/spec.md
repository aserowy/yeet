## ADDED Requirements

### Requirement: No explanatory comments in source code

Source code SHALL NOT contain one-line comments that merely explain what the following lines of code do. If intent is unclear from the code itself, the code SHALL be refactored (e.g., extracted into a well-named function) rather than annotated with a comment.

Permitted comment types: TODO, NOTE, FIX, HACK, FIXME, SAFETY markers.

#### Scenario: Explanatory comment removed
- **WHEN** a source file contains a comment like `// Build read-only buffer metadata object` followed by code that does exactly that
- **THEN** the comment SHALL be removed
- **THEN** the code SHALL remain unchanged

#### Scenario: TODO marker preserved
- **WHEN** a source file contains `// TODO: change to tokio fs`
- **THEN** the comment SHALL be preserved
