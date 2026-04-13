## 1. Fix Line Number Width

- [x] 1.1 Remove the trailing space from the absolute non-cursor line number format string in `yeet-buffer/src/view/prefix.rs` (change `"{:>width$} "` to `"{:>width$}"`)
- [x] 1.2 Add a unit test verifying that in absolute mode the cursor line number and non-cursor line number produce the same number of visible characters

## 2. Validation

- [x] 2.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [x] 2.2 Run `cargo test` and fix any failing tests
- [x] 2.3 Run `git add -A && nix build .` and fix any build errors
