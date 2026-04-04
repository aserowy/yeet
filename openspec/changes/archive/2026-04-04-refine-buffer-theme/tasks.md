## 1. BufferTheme Struct Refactor

- [x] 1.1 Add `color_to_ansi_fg` and `color_to_ansi_bg` helper functions to `yeet-buffer/src/view/` (duplicate from frontend, buffer crate has no frontend dependency)
- [x] 1.2 Define cursor mode and reset constants in `yeet-buffer/src/view/` (`CURSOR_NORMAL_CODE`, `CURSOR_NORMAL_RESET`, `CURSOR_INSERT_CODE`, `CURSOR_INSERT_RESET`, `CURSOR_LINE_RESET`)
- [x] 1.3 Change `BufferTheme` fields in `yeet-buffer/src/lib.rs` to all `Color`: remove `cursor_line_reset`, `cursor_normal_code/reset`, `cursor_insert_code/reset`, `border_fg` (String); rename `border_fg_color`→`border_fg`, `border_bg_color`→`border_bg`; change `cursor_line_bg`, `search_bg`, `line_nr` from `String` to `Color`; rename `cur_line_nr_bold` to `cur_line_nr` as `Color`

## 2. Buffer View Updates

- [x] 2.1 Update `yeet-buffer/src/view/line.rs` to convert `Color` fields to ANSI strings using the new helpers and use the defined constants for cursor mode/reset codes
- [x] 2.2 Update `yeet-buffer/src/view/prefix.rs` to convert `Color` fields to ANSI strings using the new helpers
- [x] 2.3 Update `yeet-buffer/src/view/mod.rs` to use `border_fg`/`border_bg` (`Color`) fields and update the test helper `test_theme()` for the new struct shape

## 3. Frontend Theme Updates

- [x] 3.1 Update `Theme::to_buffer_theme()` and `Theme::to_buffer_theme_with_border()` in `yeet-frontend/src/theme.rs` to return `Color` values instead of ANSI strings
- [x] 3.2 Update all theme tests in `yeet-frontend/src/theme.rs` for the new `BufferTheme` field names and types

## 4. from_enumeration Refactor

- [x] 4.1 Change `from_enumeration` in `yeet-frontend/src/update/enumeration.rs` to accept `&Theme` instead of two ANSI string parameters
- [x] 4.2 Update `set_directory_content` to pass `&Theme` to `from_enumeration` (remove pre-computed ANSI strings)
- [x] 4.3 Update `from_enumeration` call in `yeet-frontend/src/update/path.rs` to pass `&Theme`
- [x] 4.4 Remove unused ANSI string parameters from `set_directory_content` and callers if no longer needed

## 5. Remove Unused Token Constants

- [x] 5.1 Remove `COMMANDLINE_FG`, `COMMANDLINE_BG`, `CURSOR_NORMAL`, `CURSOR_INSERT`, `SYNTAX_THEME` constants from `tokens` module in `yeet-frontend/src/theme.rs`
- [x] 5.2 Remove default color registrations for `COMMANDLINE_FG` and `COMMANDLINE_BG` from the `Theme::default()` impl (already absent)
- [x] 5.3 Update theme tests if any reference removed tokens (none did)

## 6. Verify

- [x] 6.1 Run `cargo check` and fix any remaining compilation errors
- [x] 6.2 Run `cargo test` and fix any test failures
