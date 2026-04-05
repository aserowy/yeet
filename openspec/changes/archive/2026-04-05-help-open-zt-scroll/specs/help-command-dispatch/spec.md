## MODIFIED Requirements

### Requirement: Open help for a specific topic
The system SHALL resolve `<topic>` against all three structural levels of help pages: page titles (`#`), section headings (`##`), and entry identifiers (`` ### `identifier` ``). The matching help page SHALL be displayed as a read-only buffer in a horizontal split below the current window, scrolled so the matching heading is at the top of the visible viewport area.

#### Scenario: Topic matches a page title
- **WHEN** the user executes `:help <topic>` where `<topic>` matches a page title (`#` heading)
- **THEN** the matching help page is opened at the beginning

#### Scenario: Topic matches a section heading
- **WHEN** the user executes `:help <topic>` where `<topic>` matches a section heading (`##`) within a help page
- **THEN** the help page containing that section is opened with the section heading positioned at the top of the viewport

#### Scenario: Topic matches an entry identifier
- **WHEN** the user executes `:help <topic>` where `<topic>` matches an entry identifier (`` ### `identifier` ``)
- **THEN** the help page containing that entry is opened with the entry heading positioned at the top of the viewport

#### Scenario: Topic not found
- **WHEN** the user executes `:help <topic>` where `<topic>` does not match any page, section, or entry
- **THEN** the system SHALL display an error message indicating the topic was not found
