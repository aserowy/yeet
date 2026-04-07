## ADDED Requirements

### Requirement: Tests must not assume home directory exists on disk

Tests that fall back to `dirs::home_dir()` SHALL verify that the returned path exists on disk before asserting success-path behavior. When the home directory does not exist (e.g., Nix build sandbox), the test SHALL assert the error path instead.

#### Scenario: Home directory exists on disk

- **WHEN** a test invokes a command that falls back to the home directory
- **AND** `dirs::home_dir()` returns a path that exists on disk
- **THEN** the test SHALL assert the success path (e.g., `NavigateToPath` action)

#### Scenario: Home directory does not exist on disk

- **WHEN** a test invokes a command that falls back to the home directory
- **AND** `dirs::home_dir()` returns a path that does not exist on disk (e.g., `/homeless-shelter` in Nix sandbox)
- **THEN** the test SHALL assert the error path (e.g., command error message)

#### Scenario: Home directory is not available

- **WHEN** a test invokes a command that falls back to the home directory
- **AND** `dirs::home_dir()` returns `None`
- **THEN** the test SHALL assert the error path
