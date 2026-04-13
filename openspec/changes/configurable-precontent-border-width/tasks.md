## 1. Add Optional Override Field

- [ ] 1.1 Add `precontent_border_width: Option<usize>` field to `ViewPort` in `yeet-buffer/src/model/viewport.rs` with default `None`
- [ ] 1.2 Update `get_precontent_border_width()` to return the override value when `Some(n)`, falling back to computed value when `None`

## 2. Set Override on Commandline

- [ ] 2.1 In `yeet-frontend/src/model/mod.rs`, set `precontent_border_width: Some(0)` on the `CommandLine::default()` viewport

## 3. Tests

- [ ] 3.1 Add a test verifying `get_precontent_border_width()` returns 0 when `precontent_border_width` is `Some(0)` and precontent width > 0
- [ ] 3.2 Add a test verifying `get_precontent_border_width()` returns the override value when `precontent_border_width` is `Some(n)` regardless of precontent width
- [ ] 3.3 Add a test verifying `get_precontent_border_width()` returns the computed value when `precontent_border_width` is `None`

## 4. Validation

- [ ] 4.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [ ] 4.2 Run `cargo test` and fix any failing tests
- [ ] 4.3 Run `git add -A && nix build .` and fix any build errors
