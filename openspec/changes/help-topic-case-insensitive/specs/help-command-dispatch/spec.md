## MODIFIED Requirements

### Requirement: Open help for a specific topic
The system SHALL resolve `<topic>` against all three structural levels of help pages: page titles (`#`), section headings (`##`), and entry identifiers (`` ### `identifier` ``). Topic matching SHALL be case-insensitive. The matching help page SHALL be displayed as a read-only buffer in a horizontal split below the current window, scrolled so the matching heading is at the top of the visible viewport area.

#### Scenario: Topic matches a section heading with different casing
- **WHEN** the user executes `:help file operations` where "File Operations" is a `##` heading
- **THEN** the help page containing that section is opened, scrolled to the section

#### Scenario: Topic matches an entry identifier with different casing
- **WHEN** the user executes `:help Split` where `split` is an entry identifier
- **THEN** the help page containing that entry is opened, scrolled to the entry

#### Scenario: Topic matches a page name with different casing
- **WHEN** the user executes `:help Commands` where `commands` is a page name
- **THEN** the matching help page is opened at the beginning

#### Scenario: Topic not found
- **WHEN** the user executes `:help <topic>` where `<topic>` does not match any page, section, or entry regardless of casing
- **THEN** the system SHALL display an error message indicating the topic was not found
