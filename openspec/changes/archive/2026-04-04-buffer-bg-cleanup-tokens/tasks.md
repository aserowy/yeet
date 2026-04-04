## 1. Remove unused tokens

- [x] 1.1 Delete `COMMANDLINE_FG`, `COMMANDLINE_BG` constants from the `tokens` module in `yeet-frontend/src/theme.rs`
- [x] 1.2 Remove the corresponding `colors.insert(...)` lines from `Theme::default()` (none existed)
- [x] 1.3 Remove any test assertions referencing the deleted tokens (none existed)

## 2. Add BufferBg token

- [x] 2.1 Add `BUFFER_BG` constant to the `tokens` module with value `"BufferBg"`
- [x] 2.2 Insert `Color::Reset` default for `BUFFER_BG` in `Theme::default()`
- [x] 2.3 Add `buffer_bg` field to the `BufferTheme` struct in `yeet-buffer/src/lib.rs`
- [x] 2.4 Set `buffer_bg` in `to_buffer_theme()` and `to_buffer_theme_with_border()` in `yeet-frontend/src/theme.rs`

## 3. Wire BufferBg into rendering

- [x] 3.1 Apply `buffer_bg` as the background style in the buffer `Paragraph` rendering in `yeet-buffer/src/view/mod.rs`
- [x] 3.2 Prepend `buffer_bg` ANSI bg escape to non-cursor lines in `add_line_styles` (`yeet-buffer/src/view/line.rs`)
- [x] 3.3 Replace bare `\x1b[0m` resets in line number rendering (`prefix.rs:get_line_number`) with reset + `buffer_bg` re-apply
- [x] 3.4 Replace bare `\x1b[0m` resets in sign rendering (`prefix.rs:get_signs`) with reset + `buffer_bg` re-apply
- [x] 3.5 Replace bare `\x1b[0m` reset in search highlight (`line.rs:add_search_styles`) with reset + `buffer_bg` re-apply
- [x] 3.6 Replace `CURSOR_LINE_RESET` (`\x1b[0m`) in cursor line styling (`line.rs:add_cursor_styles`) with reset + `buffer_bg` re-apply

## 4. Remove to_buffer_theme and fix border token usage

- [x] 4.1 Delete `to_buffer_theme()` method from `Theme` in `yeet-frontend/src/theme.rs`
- [x] 4.2 Replace `theme.to_buffer_theme()` in `buffer.rs:214` (split pane branch) with `theme.to_buffer_theme_with_border(tokens::SPLIT_BORDER_FG, tokens::SPLIT_BORDER_BG)`
- [x] 4.3 Replace `theme.to_buffer_theme()` in `commandline.rs:12` with `theme.to_buffer_theme_with_border(tokens::SPLIT_BORDER_FG, tokens::SPLIT_BORDER_BG)`
- [x] 4.4 Update tests in `theme.rs` that call `to_buffer_theme()` to use `to_buffer_theme_with_border(tokens::SPLIT_BORDER_FG, tokens::SPLIT_BORDER_BG)`

## 5. Fix split border propagation in directory windows

- [x] 5.1 In `Window::Directory` branch of `render_window` in `buffer.rs`, set `draw_borders: None` on the dir_context for parent and current panes so they use their viewport's own `show_border`
- [x] 5.2 For the preview pane, when parent context has `draw_borders: Some(true)`, use `is_directory_pane: false` and `draw_borders: Some(true)` so it gets split border colors
- [x] 5.3 Add test: directory window in vertical split — preview pane renders with split border context, not directory border context

## 6. Wire sign tokens into sign generation

- [x] 6.1 Change `generate_sign` in `sign.rs` to take `&Theme` and use `theme.sign_qfix_style()`/`theme.sign_mark_style()` instead of hardcoded ANSI codes; remove `generate_sign_with_styles`
- [x] 6.2 Update `set()` in `sign.rs` to take `&Theme` and pass it to `generate_sign`
- [x] 6.3 Update `set_sign_if_qfix` and `set_sign_if_marked` to take `&Theme` and pass it to `set()`
- [x] 6.4 Update `set_sign_for_paths` and `set_sign_for_paths_in_buffer` to take `&Theme` and pass it to `set()`
- [x] 6.5 Thread `&Theme` through callers: `enumeration.rs`, `path.rs`, `mark.rs`, `qfix.rs`, `command/qfix.rs`, `command/mod.rs`, `update/mod.rs`
- [x] 6.6 Add test that `generate_sign` uses theme sign styles

## 7. Tests

- [x] 7.1 Add test for `BufferBg` default value (`Color::Reset`)
- [x] 7.2 Add test that `to_buffer_theme_with_border()` includes `buffer_bg`
- [x] 7.3 Verify all existing tests pass after token removal
- [x] 7.4 Verify all tests pass after ANSI reset changes
- [x] 7.5 Verify all tests pass after to_buffer_theme removal
- [x] 7.6 Verify all tests pass after split border fix
- [x] 7.7 Verify all tests pass after sign theme wiring
