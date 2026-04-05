## MODIFIED Requirements

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
