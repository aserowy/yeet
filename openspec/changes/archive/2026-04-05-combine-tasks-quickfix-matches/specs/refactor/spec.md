## ADDED Requirements

### Requirement: Combined match patterns for Tasks and QuickFix window variants
Where `Window::Tasks` and `Window::QuickFix` match arms have identical bodies, they SHALL be combined using `|` patterns.

#### Scenario: Duplicate arms are combined
- **WHEN** a match statement has separate `Window::Tasks` and `Window::QuickFix` arms with the same body
- **THEN** they SHALL be combined into a single `Window::QuickFix(vp) | Window::Tasks(vp) =>` arm
