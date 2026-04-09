## 1. Plugin Vendoring and Loading

- [ ] 1.1 Add `git@github.com:aserowy/yeet-directory-icons.git` as a git submodule at `plugins/directory-icons`
- [ ] 1.2 Wire startup/plugin loading so the vendored directory-icons plugin is available to directory rendering
- [ ] 1.3 Add startup error reporting for missing `plugins/directory-icons` path

## 2. Icon Resolution Integration

- [ ] 2.1 Define the icon descriptor interface consumed by directory rendering (glyph + theme token key)
- [ ] 2.2 Integrate plugin icon lookup by filename/extension with fallback behavior on unknown or resolver error
- [ ] 2.3 Add tests for `name.rs` and unknown-extension icon resolution scenarios

## 3. Directory Buffer Rendering and Cursor Semantics

- [ ] 3.1 Extend directory line prefix rendering to include a fixed-width icon column between line numbers and filename text
- [ ] 3.2 Ensure wrapped continuation lines preserve prefix alignment and do not duplicate icon column content
- [ ] 3.3 Update cursor/edit-column mapping so Normal/Insert edits remain filename-only and icon column is non-editable
- [ ] 3.4 Add buffer-view tests for cursor start position at filename start with icon column present

## 4. Theme Tokens for Icon Colors

- [ ] 4.1 Add directory icon class tokens and defaults to the theme token registry
- [ ] 4.2 Expose icon tokens through Lua theme assignment (`y.theme.<TokenName>`) with existing hex parsing/fallback behavior
- [ ] 4.3 Map icon descriptor token keys to theme colors with a fallback token for unmapped classes
- [ ] 4.4 Add theming tests for token override, default behavior, and unknown-class fallback

## 5. Validation and Documentation

- [ ] 5.1 Add/update user-facing docs under `docs/help/` for directory icons setup, token customization, and cursor behavior
- [ ] 5.2 Run required checks: `markdownlint` (docs markdown), `cargo fmt`, `cargo clippy`, `cargo test`, and `git add -A && nix build .`
- [ ] 5.3 Address all check failures and finalize implementation readiness for review
