### Requirement: BufferTheme has no default construction
The `yeet_buffer::BufferTheme` struct SHALL NOT implement `Default`. Callers MUST provide all field values explicitly when constructing a `BufferTheme`. All fields SHALL be of type `ratatui::style::Color`. The struct SHALL NOT contain ANSI escape code strings, reset sequences, or cursor mode codes.

#### Scenario: Compile-time enforcement
- **WHEN** a consumer attempts `BufferTheme::default()`
- **THEN** the code fails to compile with a missing trait implementation error

#### Scenario: All fields are Color type
- **WHEN** inspecting the `BufferTheme` struct definition
- **THEN** every field is of type `ratatui::style::Color` with no `String` fields

#### Scenario: No cursor mode or reset fields
- **WHEN** inspecting the `BufferTheme` struct definition
- **THEN** there are no fields for cursor normal/insert codes, cursor line reset, or any ANSI escape sequences

### Requirement: Single view entry point requires a theme
The `yeet_buffer` crate SHALL expose a single public `view()` function whose signature includes a `&BufferTheme` parameter. There SHALL NOT be a convenience variant that supplies a default theme internally.

#### Scenario: Buffer view with explicit theme
- **WHEN** `yeet_buffer::view(viewport, mode, buffer, theme, frame)` is called with a valid `BufferTheme`
- **THEN** the buffer is rendered using the colors from the provided theme

#### Scenario: No implicit-default view function exists
- **WHEN** a consumer attempts to call a `view()` function without a `&BufferTheme` parameter
- **THEN** the code fails to compile because no such function signature exists

### Requirement: Frontend theme is the sole BufferTheme factory
The `yeet_frontend::theme::Theme::to_buffer_theme()` method SHALL be the only production construction site for `BufferTheme`. It SHALL populate every field from the centralized token map. Fields SHALL be `Color` values resolved directly from theme tokens, not ANSI string conversions.

#### Scenario: All BufferTheme fields derived from theme tokens
- **WHEN** `Theme::to_buffer_theme()` is called
- **THEN** the returned `BufferTheme` contains `Color` values resolved from the token map

#### Scenario: BufferTheme fields map to tokens
- **WHEN** `Theme::to_buffer_theme()` is called with default theme
- **THEN** `cursor_line_bg` equals `Color::Rgb(128, 128, 128)`, `search_bg` equals `Color::Red`, `line_nr` equals `Color::Rgb(128, 128, 128)`, `cur_line_nr` equals `Color::White`, `border_fg` equals `Color::Black`, `border_bg` equals `Color::Reset`
