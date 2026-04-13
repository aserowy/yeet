## 1. Fix ANSI State Bleeding in Prefix Components

- [ ] 1.1 In `get_signs()` in `yeet-buffer/src/view/prefix.rs`, ensure the returned `Ansi` string ends with an explicit `ansi_reset_with_bg(theme.buffer_bg)` when `sign_column_width > 0`, so that sign styling does not bleed into the line number component
- [ ] 1.2 In `get_line_number()` in `yeet-buffer/src/view/prefix.rs`, ensure all code paths (cursor, non-cursor absolute, non-cursor relative) end with an explicit `ansi_reset_with_bg(theme.buffer_bg)` when `line_number != None`, so that line number styling does not bleed into the prefix column component
- [ ] 1.3 In `get_prefix_column()` in `yeet-buffer/src/view/prefix.rs`, ensure the returned `Ansi` string ends with an explicit `ansi_reset_with_bg(theme.buffer_bg)` when `prefix_column_width > 0`, so that prefix column styling does not bleed into the border or content

## 2. Add Tests for ANSI Isolation

- [ ] 2.1 Add a test in `yeet-buffer/src/view/prefix.rs` that verifies `get_signs()` output ends with an ANSI reset when sign column width is non-zero
- [ ] 2.2 Add a test in `yeet-buffer/src/view/prefix.rs` that verifies `get_line_number()` output for non-cursor absolute mode ends with an ANSI reset (currently it emits no ANSI at all)
- [ ] 2.3 Add a test in `yeet-buffer/src/view/mod.rs` that verifies the `ansi_to_tui` parsed spans for a prefix column icon do not inherit bold or color from the preceding line number span

## 3. Validation

- [ ] 3.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [ ] 3.2 Run `cargo test` and fix any failing tests
- [ ] 3.3 Run `git add -A && nix build .` and fix any build errors
