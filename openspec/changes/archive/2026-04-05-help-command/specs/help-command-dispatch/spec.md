## ADDED Requirements

### Requirement: Open help index with bare help command
The system SHALL open a help index page when the user executes the `:help` command without arguments. The help index SHALL be displayed as a read-only buffer in a horizontal split below the current window.

#### Scenario: User runs :help with no arguments
- **WHEN** the user executes `:help`
- **THEN** a horizontal split is created with the help index page displayed in the bottom pane as a read-only buffer

#### Scenario: Help split receives focus
- **WHEN** the help buffer opens in a horizontal split
- **THEN** the help pane SHALL receive focus

### Requirement: Open help for a specific topic
The system SHALL resolve `<topic>` against all three structural levels of help pages: page titles (`#`), section headings (`##`), and entry identifiers (`` ### `identifier` ``). The matching help page SHALL be displayed as a read-only buffer in a horizontal split below the current window, scrolled to the matching location.

#### Scenario: Topic matches a page title
- **WHEN** the user executes `:help <topic>` where `<topic>` matches a page title (`#` heading)
- **THEN** the matching help page is opened at the beginning

#### Scenario: Topic matches a section heading
- **WHEN** the user executes `:help <topic>` where `<topic>` matches a section heading (`##`) within a help page
- **THEN** the help page containing that section is opened, scrolled to the section

#### Scenario: Topic matches an entry identifier
- **WHEN** the user executes `:help <topic>` where `<topic>` matches an entry identifier (`` ### `identifier` ``)
- **THEN** the help page containing that entry is opened, scrolled to the entry

#### Scenario: Topic not found
- **WHEN** the user executes `:help <topic>` where `<topic>` does not match any page, section, or entry
- **THEN** the system SHALL display an error message indicating the topic was not found
