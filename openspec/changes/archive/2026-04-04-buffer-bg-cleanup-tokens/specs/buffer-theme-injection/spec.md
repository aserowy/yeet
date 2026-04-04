## MODIFIED Requirements

### Requirement: Frontend theme is the sole BufferTheme factory
The `yeet_frontend::theme::Theme` SHALL expose only `to_buffer_theme_with_border(fg_token, bg_token)` as the construction method for `BufferTheme`. The convenience method `to_buffer_theme()` SHALL NOT exist. All call sites MUST explicitly specify which border fg/bg tokens to use.

#### Scenario: All BufferTheme fields derived from theme tokens
- **WHEN** `Theme::to_buffer_theme_with_border(fg, bg)` is called
- **THEN** the returned `BufferTheme` contains `Color` values resolved from the token map, with `border_fg` and `border_bg` from the specified tokens

#### Scenario: BufferTheme fields map to tokens
- **WHEN** `Theme::to_buffer_theme_with_border(SPLIT_BORDER_FG, SPLIT_BORDER_BG)` is called with default theme
- **THEN** `cursor_line_bg` equals `Color::Rgb(128, 128, 128)`, `search_bg` equals `Color::Red`, `line_nr` equals `Color::Rgb(128, 128, 128)`, `cur_line_nr` equals `Color::White`, `border_fg` equals `Color::Black`, `border_bg` equals `Color::Reset`

#### Scenario: No convenience method with implicit border tokens
- **WHEN** a consumer attempts to call `theme.to_buffer_theme()` without border token arguments
- **THEN** the code fails to compile because no such method exists

#### Scenario: Directory pane uses directory border tokens
- **WHEN** a directory pane constructs its `BufferTheme`
- **THEN** it calls `to_buffer_theme_with_border(DIRECTORY_BORDER_FG, DIRECTORY_BORDER_BG)`

#### Scenario: Split pane uses split border tokens
- **WHEN** a split pane constructs its `BufferTheme`
- **THEN** it calls `to_buffer_theme_with_border(SPLIT_BORDER_FG, SPLIT_BORDER_BG)`

#### Scenario: Directory window inside a vertical split uses split border for separator
- **WHEN** a directory window is the first child of a vertical split
- **THEN** the preview pane's border (the split separator) uses `SPLIT_BORDER_FG`/`SPLIT_BORDER_BG`, not `DIRECTORY_BORDER_FG`/`DIRECTORY_BORDER_BG`

#### Scenario: Directory panes do not inherit draw_borders from split context
- **WHEN** a directory window is inside a vertical split with `draw_borders: Some(true)`
- **THEN** the parent and current panes use their own viewport `show_border` setting, not the split's `draw_borders` override
