## 1. Window Model

- [x] 1.1 Add `set_wrap(bool)` method on `Window` enum in `yeet-frontend/src/model/mod.rs` that sets the `wrap` field on all viewports within the window variant (all three for Directory, single for QuickFix/Tasks/Help, recursive for splits)

## 2. Command Dispatch

- [x] 2.1 Add `("set", args)` match arm in `yeet-frontend/src/update/command/mod.rs` that parses `wrap`/`nowrap` arguments
- [x] 2.2 Call `set_wrap(true)` for `wrap` and `set_wrap(false)` for `nowrap` on the focused window
- [x] 2.3 Return `Action::EmitMessages` with error for unknown or empty `:set` arguments

## 3. Tests

- [x] 3.1 Add tests for `set_wrap` on Directory window (all three viewports toggled)
- [x] 3.2 Add tests for `set_wrap` on single-viewport windows
- [x] 3.3 Add tests for `:set wrap`, `:set nowrap`, and `:set <invalid>` command dispatch

## 4. Documentation

- [x] 4.1 Add `:set wrap` and `:set nowrap` to the relevant docs markdown file

## 5. Validation

- [x] 5.1 Run `cargo test`, `cargo clippy`, and `cargo fmt` and fix any issues
- [x] 5.2 Run `markdownlint` on updated docs and fix any issues
