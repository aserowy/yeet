### Requirement: BufferTheme is created at point of use
Functions that call `yeet_buffer::view()` SHALL create the `BufferTheme` locally via `theme.to_buffer_theme()` rather than receiving it as a threaded parameter from parent functions.

#### Scenario: Intermediate layout functions do not accept BufferTheme
- **WHEN** `render_window` is called to lay out split or directory windows
- **THEN** its signature does not include a `BufferTheme` parameter

#### Scenario: Leaf rendering functions create BufferTheme locally
- **WHEN** `render_buffer_slot` or `render_directory_buffer` needs to call `yeet_buffer::view()`
- **THEN** it creates a `BufferTheme` from the `&Theme` parameter and passes it to the view call
