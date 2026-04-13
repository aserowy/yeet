## 1. Fix Border Width Condition

- [ ] 1.1 In `get_border_width()` in `yeet-buffer/src/model/viewport.rs`, change the condition from `self.get_prefix_width() > 0` to `self.get_prefix_width() > 0 || self.prefix_column_width > 0` so the border is rendered when any pre-content column is active

## 2. Update Tests

- [ ] 2.1 Update the `offset_width_includes_prefix_column` test in `yeet-buffer/src/model/viewport.rs` to expect border width of 1 (offset = 0 + 1 + 2 = 3) instead of 0 + 0 + 2 = 2
- [ ] 2.2 Update the `content_width_reduced_by_prefix_column` test in `yeet-buffer/src/model/viewport.rs` to account for the additional 1-cell border when prefix column is active
- [ ] 2.3 Add a test in `yeet-buffer/src/model/viewport.rs` that verifies `get_border_width()` returns 1 when only `prefix_column_width > 0` and signs/line numbers are zero
- [ ] 2.4 Add a test in `yeet-buffer/src/model/viewport.rs` that verifies `get_border_width()` returns 0 when all pre-content column widths are zero

## 3. Validation

- [ ] 3.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [ ] 3.2 Run `cargo test` and fix any failing tests
- [ ] 3.3 Run `git add -A && nix build .` and fix any build errors
