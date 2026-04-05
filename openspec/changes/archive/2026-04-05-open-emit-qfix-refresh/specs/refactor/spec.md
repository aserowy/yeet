## ADDED Requirements

### Requirement: Open handler uses message-based refresh
The quickfix Enter handler in `open.rs` SHALL emit `Message::QuickFixChanged` instead of calling `refresh_quickfix_buffer_in_window` directly, ensuring cross-tab refresh.

#### Scenario: Enter refreshes all tabs
- **WHEN** the user presses Enter on a copen entry
- **THEN** all copen buffers across all tabs SHALL be refreshed
