## 1. Token Registration

- [ ] 1.1 Rename `BorderFg` to `SplitBorderFg` and add new token constants to `yeet-frontend/src/theme.rs`: `BufferFileFg`, `BufferDirectoryFg`, `StatusLinePermissionsFg`, `StatusLineBorderBg`, `DirectoryBorderFg`, `DirectoryBorderBg`, `SplitBorderBg`
- [ ] 1.2 Register default colors for all new tokens in the `Theme` default implementation

## 2. BufferTheme Extension

- [ ] 2.1 Add `border_fg_color` and `border_bg_color` fields (as `ratatui::Color`) to `BufferTheme` in `yeet-buffer/src/lib.rs`
- [ ] 2.2 Update `Theme::to_buffer_theme()` in `yeet-frontend/src/theme.rs` to populate the new `BufferTheme` fields from `SplitBorderFg`/`DirectoryBorderFg` tokens depending on context
- [ ] 2.3 Replace hardcoded `Color::Black` in `yeet-buffer/src/view/mod.rs` with `BufferTheme` border color fields

## 3. Buffer Entry Foreground Colors

- [ ] 3.1 Update `from_enumeration` in `yeet-frontend/src/update/enumeration.rs` to accept theme-derived ANSI color strings for file and directory foreground
- [ ] 3.2 Replace hardcoded `\x1b[94m` directory color with `BufferDirectoryFg` theme token ANSI code
- [ ] 3.3 Wrap file entries with `BufferFileFg` theme token ANSI code
- [ ] 3.4 Update all call sites of `from_enumeration` to pass theme colors

## 4. Statusline Enhancements

- [ ] 4.1 Apply `StatusLinePermissionsFg` theme token to permissions text in `filetree_status` function in `yeet-frontend/src/view/statusline.rs`
- [ ] 4.2 Apply `StatusLineBorderBg` theme token to the statusline border style (add background to existing `StatusLineBorderFg` style)

## 5. Directory Window and Split Border Colors

- [ ] 5.1 Pass border token information through `RenderContext` in `yeet-frontend/src/view/buffer.rs` so directory windows use `DirectoryBorder*` tokens and vertical splits use `SplitBorder*` tokens (formerly `BorderFg`)
- [ ] 5.2 Create separate `BufferTheme` instances (or pass border colors) for directory window panes vs split panes with the appropriate token colors

## 6. Spec Updates

- [ ] 6.1 Write spec tests for new token defaults matching current hardcoded appearance
- [ ] 6.2 Write spec tests for buffer entry foreground color application
- [ ] 6.3 Write spec tests for statusline permissions and border background styling
