## ADDED Requirements

### Requirement: BufferTheme has no default construction
The `yeet_buffer::BufferTheme` struct SHALL NOT implement `Default`. Callers MUST provide all field values explicitly when constructing a `BufferTheme`.

#### Scenario: Compile-time enforcement
- **WHEN** a consumer attempts `BufferTheme::default()`
- **THEN** the code fails to compile with a missing trait implementation error

### Requirement: Single view entry point requires a theme
The `yeet_buffer` crate SHALL expose a single public `view()` function whose signature includes a `&BufferTheme` parameter. There SHALL NOT be a convenience variant that supplies a default theme internally.

#### Scenario: Buffer view with explicit theme
- **WHEN** `yeet_buffer::view(viewport, mode, buffer, theme, frame)` is called with a valid `BufferTheme`
- **THEN** the buffer is rendered using the colors from the provided theme

#### Scenario: No implicit-default view function exists
- **WHEN** a consumer attempts to call a `view()` function without a `&BufferTheme` parameter
- **THEN** the code fails to compile because no such function signature exists

### Requirement: Frontend theme is the sole BufferTheme factory
The `yeet_frontend::theme::Theme::to_buffer_theme()` method SHALL be the only production construction site for `BufferTheme`. It SHALL populate every field from the centralized token map.

#### Scenario: All BufferTheme fields derived from theme tokens
- **WHEN** `Theme::to_buffer_theme()` is called
- **THEN** the returned `BufferTheme` contains ANSI codes derived from the token values in the `Theme`, not from hardcoded literals (except for mode-invariant reset/modifier codes)

## REMOVED Requirements

### Requirement: BufferTheme default implementation
**Reason**: Default ANSI values duplicated color knowledge that belongs in the centralized theme system. Removing the default enforces that all rendering goes through the theme pipeline.
**Migration**: Replace any `BufferTheme::default()` calls with `theme.to_buffer_theme()` using a `Theme` instance.

### Requirement: Convenience view function without theme parameter
**Reason**: The non-themed `view()` wrapper was unused in practice and allowed bypassing the theme system.
**Migration**: Use the new `view()` (previously `view_themed()`) which requires a `&BufferTheme` parameter.
