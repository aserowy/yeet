## 1. Add Optional Override Field

- [x] 1.1 Add `precontent_border_width: Option<usize>` field to `ViewPort` in `yeet-buffer/src/model/viewport.rs` with default `None`
- [x] 1.2 Update `get_precontent_border_width()` to return the override value when `Some(n)`, falling back to computed value when `None`

## 2. Set Override on Commandline

- [x] 2.1 In `yeet-frontend/src/model/mod.rs`, set `precontent_border_width: Some(0)` on the `CommandLine::default()` viewport

## 3. Tests

- [x] 3.1 Add a test verifying `get_precontent_border_width()` returns 0 when `precontent_border_width` is `Some(0)` and precontent width > 0
- [x] 3.2 Add a test verifying `get_precontent_border_width()` returns the override value when `precontent_border_width` is `Some(n)` regardless of precontent width
- [x] 3.3 Add a test verifying `get_precontent_border_width()` returns the computed value when `precontent_border_width` is `None`

## 4. Validation

- [x] 4.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [x] 4.2 Run `cargo test` and fix any failing tests
- [x] 4.3 Run `git add -A && nix build .` and fix any build errors
