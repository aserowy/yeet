## 1. Replace icon column with prefix column in yeet-buffer

- [x] 1.1 Rename `icon_column_width` to `prefix_column_width` in `ViewPort` struct (`yeet-buffer/src/model/viewport.rs`), update `get_prefix_width()` to use the new field, and update all tests in the file
- [x] 1.2 Remove the `icon` field from `BufferLine` struct (`yeet-buffer/src/model/mod.rs`)
- [x] 1.3 Replace `get_icon_column()` in `yeet-buffer/src/view/prefix.rs` with `get_prefix_column()` that renders the `prefix` field right-aligned within `prefix_column_width`, falling back to spaces when prefix is None
- [x] 1.4 Update both wrap and no-wrap rendering paths in `yeet-buffer/src/view/mod.rs` to call `get_prefix_column()` instead of `get_icon_column()` and `get_custom_prefix()` separately
- [x] 1.5 Update `get_offset_width()` in viewport.rs to account for `prefix_column_width` instead of separate icon column and custom prefix width calculations
- [x] 1.6 Update all existing tests in `yeet-buffer/src/view/prefix.rs` and `yeet-buffer/src/view/mod.rs` to use `prefix_column_width` and `prefix` instead of `icon_column_width` and `icon`

## 2. Update Lua bridge for prefix column

- [x] 2.1 Rename `icon_column_width` to `prefix_column_width` in `yeet-lua/src/viewport.rs` Lua field read/write
- [x] 2.2 Remove `icon` field handling from `on_bufferline_mutate` hook context in `yeet-lua/src/hook.rs` (remove icon write-to-context and read-back)
- [x] 2.3 Update all Lua hook tests in `yeet-lua/src/hook.rs` that reference `icon` or `icon_column_width`

## 3. Fix hook firing order in frontend

- [x] 3.1 In `yeet-frontend/src/update/enumeration.rs`, move `invoke_on_bufferline_mutate` calls to fire AFTER `set_sign_if_marked` and `set_sign_if_qfix` in all code paths (`set_directory_content` and any other functions)
- [x] 3.2 Audit all other locations where `on_bufferline_mutate` fires (content buffers, help, quickfix, tasks) and ensure hooks fire after all sign operations

## 4. Set per-buffer prefix column width defaults

- [x] 4.1 Ensure `prefix_column_width` defaults to `0` on `ViewPort` (via `Default` derive) for all buffer types
- [x] 4.2 Set `prefix_column_width` to `1` on the commandline buffer viewport initialization (`yeet-frontend/src/update/commandline.rs` or wherever commandline viewport is created)

## 5. Move plugin help page discovery to yeet-plugin

- [x] 5.1 Add `help_pages` field (Vec of help page data) to `PluginSpec` in `yeet-plugin/src/spec.rs`
- [x] 5.2 Implement help page discovery function in `yeet-plugin` that scans a plugin's `docs/help/*.md` directory and returns resolved help page paths with content
- [x] 5.3 Integrate help page discovery into plugin spec initialization so `help_pages` is populated when specs are read from Lua
- [x] 5.4 Refactor `discover_plugin_help_pages()` in `yeet-frontend/src/update/command/help.rs` to read help pages from `PluginSpec.help_pages` instead of scanning the filesystem

## 6. Update directory-icons plugin

- [x] 6.1 Update `on_window_create` hook in `plugins/directory-icons/init.lua` to set `prefix_column_width = 2` instead of `icon_column_width = 1`
- [x] 6.2 Update `on_bufferline_mutate` hook in `plugins/directory-icons/init.lua` to write icon glyphs to `ctx.prefix` instead of `ctx.icon`
- [x] 6.3 Update the plugin's `docs/help/directory-icons.md` to reflect prefix-based rendering and `prefix_column_width` instead of `icon_column_width`

## 7. Update core help documentation

- [x] 7.1 Remove plugin-specific content from `docs/help/hooks.md`: remove "Trailing Slash Convention" section, "Icon Column" section, and any references to icon-specific plugin behavior
- [x] 7.2 Update `docs/help/hooks.md` to document `prefix_column_width` instead of `icon_column_width` in the viewport settings table, and remove `icon` from the on_bufferline_mutate context fields table
- [x] 7.3 Run `markdownlint` on all updated markdown files in `docs/` and fix any warnings or errors

## 8. Verify and build

- [x] 8.1 Run `cargo fmt` and `cargo clippy` to ensure code is clean
- [x] 8.2 Run `cargo test` to ensure all tests pass
- [x] 8.3 Run `git add -A && nix build .` to ensure the full build succeeds

## 9. Consolidate BUFFER_FILE_FG and BUFFER_DIRECTORY_FG into BUFFER_FG

- [x] 9.1 Replace `BUFFER_FILE_FG` and `BUFFER_DIRECTORY_FG` token constants with a single `BUFFER_FG` in `yeet-frontend/src/theme.rs`, set default to `Color::White`
- [x] 9.2 Add `buffer_fg` field to `BufferTheme` struct in `yeet-buffer/src/lib.rs` and populate it from the `BUFFER_FG` token in `to_buffer_theme_with_border()`
- [x] 9.3 Wire `buffer_fg` into the buffer rendering pipeline (`yeet-buffer/src/view/line.rs` and `yeet-buffer/src/view/style.rs`) so unstyled content text uses `buffer_fg` as its foreground color
- [x] 9.4 Update `plugins/bluloco-theme/init.lua` to set `BufferFg` instead of `BufferFileFg`/`BufferDirectoryFg`
- [x] 9.5 Update `docs/help/theme.md` to document `BufferFg` and remove `BufferFileFg`/`BufferDirectoryFg`
- [x] 9.6 Update all tests referencing `BUFFER_FILE_FG` or `BUFFER_DIRECTORY_FG` to use `BUFFER_FG`

## 10. Suppress prefix rendering when prefix_column_width is zero

- [x] 10.1 Update `get_prefix_column()` in `yeet-buffer/src/view/prefix.rs` to return empty string when `prefix_column_width` is 0, regardless of whether `bl.prefix` has content
- [x] 10.2 Update `get_custom_prefix_width()` in `yeet-buffer/src/model/viewport.rs` to return 0 when `prefix_column_width` is 0, regardless of prefix content
- [x] 10.3 Update tests in `prefix.rs` and `viewport.rs` to reflect the new behavior (width 0 + prefix content → no rendering, no width contribution)

## 11. Final verification

- [x] 11.1 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [x] 11.2 Run `markdownlint` on updated docs
- [x] 11.3 Run `git add -A && nix build .`
