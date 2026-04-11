## 1. Plugin Loading and Mutation Hook Infrastructure

- [x] 1.1 Wire existing startup/plugin loading so user-configured `yeet-directory-icons` is available to directory rendering
- [x] 1.2 Add runtime diagnostics for `yeet-directory-icons` configuration/load failures
- [x] 1.3 Expand `on_bufferline_mutate` hook context to provide full bufferline fields (prefix, content, search_char_position, signs, icon) and buffer-type metadata (type string + path where applicable)
- [x] 1.3a Restructure `on_bufferline_mutate` hook context to use a read-only `buffer` metadata object (`ctx.buffer` with `type` and `path` fields) instead of flat `buffer_type`/`path` fields for extensibility
- [x] 1.4 Remove `is_directory` parameter from `invoke_on_bufferline_mutate` since directory-ness is now encoded in trailing slash
- [x] 1.5 Add hook invocation to content buffer population (file preview in `preview.rs`)
- [x] 1.6 Add hook invocation to help buffer population (`command/help.rs`)
- [x] 1.7 Add hook invocation to quickfix buffer population (`command/qfix/window.rs`)
- [x] 1.8 Add hook invocation to tasks buffer population (`command/task.rs`)
- [x] 1.9 Ensure deferred `PathsAdded` events (Insert mode) also defer mutation hook invocation; hooks fire on flush when events are processed after leaving Insert mode
- [x] 1.10 Register plugin `on_window_create` hook to set shared `@yeet-buffer` icon-column width to `1`

## 2. Trailing-Slash Convention and ContentKind Removal

- [x] 2.1 Add trailing slash (`/`) to directory entry names in the enumeration task runner (`task/mod.rs`) when `path.is_dir()` is true
- [x] 2.2 Add trailing slash to directory entry names in `PathsAdded` handling (`path.rs`) when `path.is_dir()` is true
- [x] 2.3 Remove `ContentKind` enum from `event.rs`
- [x] 2.4 Update `EnumerationChanged` and `EnumerationFinished` message types to use `Vec<String>` instead of `Vec<(ContentKind, String)>`
- [x] 2.5 Update `set_directory_content()` in `enumeration.rs` to determine directory-ness from trailing slash instead of `ContentKind`
- [x] 2.6 Update `update_directory_buffers_on_add()` in `path.rs` to use trailing slash convention instead of `ContentKind`
- [x] 2.7 Update sort logic in `update/mod.rs` if needed to handle trailing slashes correctly
- [x] 2.8 Update any existing tests that reference `ContentKind`

## 3. Remove icon_style and Core Styling

- [x] 3.1 Remove `icon_style` field from `BufferLine` struct in `yeet-buffer/src/model/mod.rs`
- [x] 3.2 Remove `icon_style` prepend logic from `add_line_styles()` in `yeet-buffer/src/view/line.rs`
- [x] 3.3 Remove `icon_style` prepend logic from `add_line_styles_wrap()` in `yeet-buffer/src/view/line.rs`
- [x] 3.4 Update `get_icon_column()` in `prefix.rs` to render icon glyph as-is from the `icon` field without applying `icon_style`
- [x] 3.5 Remove `icon_style` from hook read-back logic in `invoke_on_bufferline_mutate` in `hook.rs`
- [x] 3.6 Remove `icon_style` from Lua snapshot/restore in `loading.rs` if present
- [x] 3.7 Update all tests that reference `icon_style`

## 4. Bufferline Mutation Rendering

- [x] 4.1 Ensure bufferline model supports icon glyph field that the plugin can mutate via hooks
- [x] 4.2 Implement core rendering logic that reads the icon glyph from the bufferline and renders it in the icon-column prefix segment
- [x] 4.3 Update icon rendering to work without `icon_style` — icon glyph rendered as-is from the `icon` field (plugin includes ANSI sequences in the icon string)
- [x] 4.4 Add fallback behavior: if plugin does not mutate the bufferline (or hook errors), icon column remains empty and text is unchanged
- [x] 4.5 Update tests for mutation rendering without `icon_style`

## 5. Directory Buffer Rendering and Cursor Semantics

- [x] 5.1 Add icon-column support to shared `@yeet-buffer` prefix definitions so all buffer types can represent the segment consistently
- [x] 5.2 Implement shared `@yeet-buffer` icon-column rendering function with default width `0` when plugin is unavailable
- [x] 5.3 Wire directory window (three `@yeet-buffer` instances) to use the shared icon-column rendering function between line numbers and filename text
- [x] 5.4 Ensure wrapped continuation lines preserve prefix alignment and do not duplicate icon column content
- [x] 5.5 Update cursor/edit-column mapping so Normal/Insert edits remain filename-only and icon column is non-editable
- [x] 5.6 Add buffer-view tests for cursor start position at filename start with icon column present
- [x] 5.7 Add tests for width/hook behavior: icon-column width `0` by default and width `1` after plugin `on_window_create` executes

## 6. Theme Token Interaction

- [x] 6.1 Remove legacy built-in directory file/folder colorization path (no fallback — entries are plain text without plugin)
- [x] 6.2 Verify directory icon class tokens and defaults work through existing theme token registry (token names are plugin-defined)
- [x] 6.3 Verify icon tokens are exposed through Lua theme assignment (`y.theme.<TokenName>`) with existing hex parsing/fallback behavior
- [x] 6.4 Verify core token resolution with fallback for unmapped classes
- [x] 6.5 Existing theming tests cover token override, default behavior, directory-specific token, and unknown-class fallback

## 7. Plugin Update (yeet-directory-icons)

- [x] 7.1 Update plugin to check buffer type metadata and only process `directory` type buffers
- [x] 7.1a Update plugin to use `ctx.buffer.type` (metadata object) instead of `ctx.buffer_type` (flat field) for buffer type checking
- [x] 7.2 Update plugin to detect directories by trailing slash instead of `is_directory` context field
- [x] 7.3 Update plugin to style content by prepending ANSI escape sequences to the `content` field instead of setting `icon_style`
- [x] 7.4 Update plugin to include ANSI color in the `icon` string value (color prefix + glyph + reset suffix)
- [x] 7.5 Update plugin to check for existing theme token values before setting defaults (theme plugin priority)
- [x] 7.6 Strip trailing slash from filename before icon resolution (so `.git/` maps to `.git` in dir_map)

## 8. Validation and Documentation

- [x] 8.1 Update `docs/help/hooks.md` for expanded hook context (full bufferline fields, buffer-type metadata, no `is_directory`)
- [x] 8.1a Update `docs/help/hooks.md` to document the `buffer` metadata object pattern (`ctx.buffer.type`, `ctx.buffer.path`) instead of flat fields
- [x] 8.2 Update `docs/help/theme.md` for theme plugin interaction and token priority
- [x] 8.3 Run required checks: `markdownlint` (docs markdown), `cargo fmt`, `cargo clippy`, `cargo test`, and `git add -A && nix build .`
- [x] 8.3a Re-run required checks after metadata object refactoring: `cargo fmt`, `cargo clippy`, `cargo test`, and `git add -A && nix build .`
- [x] 8.4 Address all check failures and finalize implementation readiness for review

## 9. Optional Path in on_bufferline_mutate

- [x] 9.1 Change `invoke_on_bufferline_mutate` signature to accept `Option<&Path>` instead of `&Path`. Only set `ctx.buffer.path` when `Some`; omit (nil) when `None`.
- [x] 9.2 Update callers that have no meaningful path (help.rs, task.rs, qfix/window.rs) to pass `None` instead of `Path::new("")`
- [x] 9.3 Update callers that have a real path (path.rs, enumeration.rs, preview.rs) to pass `Some(&path)` / `Some(path)`
- [x] 9.4 Update `docs/help/hooks.md` to document that `ctx.buffer.path` is nil (absent) for buffer types without a path (help, quickfix, tasks)
- [x] 9.5 Run checks: `cargo fmt`, `cargo clippy`, `cargo test`

## 10. DirectoryIconsColor Theme Tokens in Plugin

- [x] 10.1 Refactor `directory-icons/init.lua` to define a `DirectoryIconsColor*` theme token for every unique color in ext_map, name_map, dir_map, and defaults. Each mapping entry references a token name instead of a raw hex color.
- [x] 10.2 In `M.setup()`, register all `DirectoryIconsColor*` token defaults — set each token on `y.theme` only if the token is not already present (theme plugins loaded first take priority).
- [x] 10.3 During bufferline mutation, resolve colors by reading `y.theme[token_name]` instead of using the hardcoded hex value from the mapping table.
- [x] 10.4 Remove the `resolve_color("BufferDirectoryFg", color)` / `resolve_color("BufferFileFg", color)` pattern; replace with per-mapping `DirectoryIconsColor*` token lookups.

## 11. DirectoryIconsColor Overrides in yeet-bluloco-theme

- [x] 11.1 Add `DirectoryIconsColor*` token overrides to `plugins/bluloco-theme/init.lua` — set all tokens the directory-icons plugin defines, using the bluloco palette colors.

## 12. Documentation Updates

- [x] 12.1 Update `docs/help/theme.md` to document `DirectoryIconsColor*` tokens, their naming convention, and how theme plugins can override them.

## 13. Final Validation

- [x] 13.1 Run full check suite: `cargo fmt`, `cargo clippy`, `cargo test`, `git add -A && nix build .`
- [x] 13.2 Commit and push submodule changes (plugins/directory-icons, plugins/bluloco-theme)

## 14. BufferType Enum

- [ ] 14.1 Introduce a `BufferType` enum in `yeet-lua/src/hook.rs` (or a shared module) with variants `Directory`, `Content`, `Help`, `Quickfix`, `Tasks`. Implement `as_str()` method mapping each variant to its lowercase string representation.
- [ ] 14.2 Change `invoke_on_bufferline_mutate` signature to accept `BufferType` instead of `&str`. Use `buffer_type.as_str()` when setting `ctx.buffer.type` in the Lua context.
- [ ] 14.3 Update all callers of `invoke_on_bufferline_mutate` to pass `BufferType::Directory`, `BufferType::Content`, `BufferType::Help`, `BufferType::Quickfix`, or `BufferType::Tasks` instead of string literals.
- [ ] 14.4 Run checks: `cargo fmt`, `cargo clippy`, `cargo test`

## 15. Plugin Help Documentation

- [ ] 15.1 Create `docs/help/directory-icons.md` in the `yeet-directory-icons` plugin (submodule) with full `DirectoryIconsColor*` token reference, usage examples, and configuration guide.
- [ ] 15.2 Create `docs/help/bluloco-theme.md` in the `yeet-bluloco-theme` plugin (submodule) with theme override documentation and palette reference.
- [ ] 15.3 Remove plugin-specific content from core `docs/help/theme.md` — remove the "Icon Tokens" section (DirectoryIconsColor token table, theme plugin priority, fallback colors) and plugin-specific references from `BufferFileFg`/`BufferDirectoryFg` descriptions.
- [ ] 15.4 Remove plugin-specific content from core `docs/help/hooks.md` — remove plugin-specific examples and references from the `on_bufferline_mutate` section (icon column plugin example, yeet-directory-icons references).

## 16. Extend :help to Discover Plugin Help Pages

- [ ] 16.1 Determine how to discover loaded plugin directories at runtime (e.g., from plugin configuration/paths stored in the Lua state or from the plugin loading infrastructure).
- [ ] 16.2 Extend the help system in `help.rs` to scan each loaded plugin's `docs/help/` directory for `*.md` files at runtime and register them as additional help pages.
- [ ] 16.3 Ensure core help pages are searched first (take priority) when resolving topics — plugin pages are only matched when no core page matches.
- [ ] 16.4 Update `docs/help/plugins.md` or `docs/help/index.md` to document that plugins can provide help pages via `docs/help/` in their plugin directory.
- [ ] 16.5 Run checks: `cargo fmt`, `cargo clippy`, `cargo test`, `git add -A && nix build .`

## 17. Final Validation (Post-Refinement)

- [ ] 17.1 Run full check suite: `cargo fmt`, `cargo clippy`, `cargo test`, `git add -A && nix build .`
- [ ] 17.2 Commit and push all changes including submodule changes
