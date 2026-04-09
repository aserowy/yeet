## 1. Plugin Loading and Mutation Hook Infrastructure

- [x] 1.1 Wire existing startup/plugin loading so user-configured `yeet-directory-icons` is available to directory rendering
- [x] 1.2 Add runtime diagnostics for `yeet-directory-icons` configuration/load failures
- [x] 1.3 Define new per-bufferline mutation hook interface: the hook receives the complete bufferline (mutable) and the given window with all metadata; the plugin directly mutates the bufferline in-place (sets icon, colors text)
- [x] 1.4 Implement mutation hook invocation in `EnumerationChanged` message handling (fires for each bufferline as directory content is progressively set)
- [x] 1.5 Implement mutation hook invocation in `EnumerationFinished` message handling (fires for each bufferline as final directory content is set)
- [x] 1.6 Implement mutation hook invocation in `PathsAdded` message handling (fires for each new bufferline added from filesystem events)
- [x] 1.7 Ensure deferred `PathsAdded` events (Insert mode) also defer mutation hook invocation; hooks fire on flush when events are processed after leaving Insert mode
- [x] 1.8 Register plugin `on_window_create` hook to set shared `@yeet-buffer` icon-column width to `1`

## 2. Bufferline Mutation Rendering

- [x] 2.1 Ensure bufferline model supports icon glyph field and text color field that the plugin can mutate via hooks
- [x] 2.2 Implement core rendering logic that reads the mutated icon glyph from the bufferline and renders it in the icon-column prefix segment
- [x] 2.3 Implement core rendering logic that reads the mutated text color from the bufferline and applies it to both icon glyph and filename text
- [x] 2.4 Add fallback behavior: if plugin does not mutate the bufferline (or hook errors), icon column remains empty and text uses default styling
- [x] 2.5 Add tests for mutation rendering, fallback, and hook-error handling

## 3. Directory Buffer Rendering and Cursor Semantics

- [x] 3.0 Add icon-column support to shared `@yeet-buffer` prefix definitions so all buffer types can represent the segment consistently
- [x] 3.1 Implement shared `@yeet-buffer` icon-column rendering function with default width `0` when plugin is unavailable
- [x] 3.2 Wire directory window (three `@yeet-buffer` instances) to use the shared icon-column rendering function between line numbers and filename text
- [x] 3.3 Ensure wrapped continuation lines preserve prefix alignment and do not duplicate icon column content
- [x] 3.4 Update cursor/edit-column mapping so Normal/Insert edits remain filename-only and icon column is non-editable
- [x] 3.5 Add buffer-view tests for cursor start position at filename start with icon column present
- [ ] 3.6 Add tests for width/hook behavior: icon-column width `0` by default and width `1` after plugin `on_window_create` executes

## 4. Theme Tokens for Icon Colors

- [ ] 4.0 Remove legacy built-in directory file/folder colorization path to avoid conflicting style sources
- [ ] 4.1 Add directory icon class tokens and defaults to the theme token registry (token names are plugin-defined; directories get their own distinct token separate from file default)
- [ ] 4.2 Expose icon tokens through Lua theme assignment (`y.theme.<TokenName>`) with existing hex parsing/fallback behavior
- [ ] 4.3 Implement core token resolution with a fallback token for unmapped classes
- [ ] 4.4 Add theming tests for token override, default behavior, directory-specific token, and unknown-class fallback

## 5. Validation and Documentation

- [ ] 5.1 Add/update user-facing docs under `docs/help/` for directory icons setup, mutation hook contract, token customization, and cursor behavior
- [ ] 5.2 Run required checks: `markdownlint` (docs markdown), `cargo fmt`, `cargo clippy`, `cargo test`, and `git add -A && nix build .`
- [ ] 5.3 Address all check failures and finalize implementation readiness for review
