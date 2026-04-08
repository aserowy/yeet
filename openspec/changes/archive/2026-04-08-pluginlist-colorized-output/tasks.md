## 1. Theme Tokens

- [x] 1.1 Add `ERROR_FG`, `WARNING_FG`, `SUCCESS_FG` constants to `tokens` module in `theme.rs`
- [x] 1.2 Add default colors for the three tokens in `Theme::default()` (red, yellow, green)
- [x] 1.3 Add `INFORMATION_FG` constant to `tokens` module in `theme.rs`
- [x] 1.4 Add default color for `INFORMATION_FG` in `Theme::default()` (blue `#8be9fd`)

## 2. PrintContent Variants

- [x] 2.1 Add `Warning(String)` variant to `PrintContent` enum in `yeet-keymap/src/message.rs`
- [x] 2.2 Update `Display` impl and any match arms for the new variant
- [x] 2.3 Add `Success(String)` variant to `PrintContent` enum
- [x] 2.4 Update `Display` impl and any match arms for the `Success` variant

## 3. Theme-Aware Commandline Rendering

- [x] 3.1 Update `commandline::print` to accept `&Theme` parameter
- [x] 3.2 Use `theme.ansi_fg(tokens::ERROR_FG)` for `PrintContent::Error` rendering
- [x] 3.3 Use `theme.ansi_fg(tokens::WARNING_FG)` for `PrintContent::Warning` rendering
- [x] 3.4 Use `theme.ansi_fg(tokens::SUCCESS_FG)` for `PrintContent::Information` rendering
- [x] 3.5 Update all callers of `commandline::print` to pass `&theme`/`&settings.theme`
- [x] 3.6 Use `theme.ansi_fg(tokens::INFORMATION_FG)` for `PrintContent::Information` rendering (replace SUCCESS_FG)
- [x] 3.7 Use `theme.ansi_fg(tokens::SUCCESS_FG)` for `PrintContent::Success` rendering

## 4. Pluginlist Color Mapping

- [x] 4.1 Update `print::plugin_list` to use `PrintContent::Success` for loaded, `PrintContent::Warning` for missing, `PrintContent::Error` for error

## 5. Build Verification

- [x] 5.1 Run `cargo fmt` and `cargo clippy` and fix all warnings
- [x] 5.2 Run `cargo test` and ensure all tests pass
- [x] 5.3 Run `nix build .` and ensure build succeeds
