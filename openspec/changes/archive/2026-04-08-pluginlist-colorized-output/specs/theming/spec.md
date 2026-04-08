## ADDED Requirements

### Requirement: Semantic status color tokens

The theme SHALL include `ErrorFg`, `WarningFg`, `SuccessFg`, and `InformationFg` color tokens for semantic coloring. These tokens SHALL be configurable via `y.theme.ErrorFg`, `y.theme.WarningFg`, `y.theme.SuccessFg`, and `y.theme.InformationFg` in `init.lua`.

#### Scenario: Default error color

- **WHEN** the user has not configured `y.theme.ErrorFg`
- **THEN** the system SHALL use red as the default error foreground color

#### Scenario: Default warning color

- **WHEN** the user has not configured `y.theme.WarningFg`
- **THEN** the system SHALL use yellow as the default warning foreground color

#### Scenario: Default success color

- **WHEN** the user has not configured `y.theme.SuccessFg`
- **THEN** the system SHALL use green as the default success foreground color

#### Scenario: Default information color

- **WHEN** the user has not configured `y.theme.InformationFg`
- **THEN** the system SHALL use blue as the default information foreground color

#### Scenario: Custom error color

- **WHEN** `init.lua` contains `y.theme.ErrorFg = '#ff0000'`
- **THEN** the system SHALL use `#ff0000` as the error foreground color in all error-colored output
