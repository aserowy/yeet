## 1. Fix continuation line indentation

- [x] 1.1 In `yeet-buffer/src/view/mod.rs` line 125, change `vp.get_offset_width(&bl) + vp.get_precontent_border_width()` to `vp.get_offset_width(&bl)`
- [x] 1.2 Add a test in `yeet-buffer/src/view/mod.rs` that verifies continuation line indentation width equals the first line's prefix width when line numbers and prefix column are active

## 2. Validation

- [x] 2.1 Run `cargo fmt` and `cargo clippy` and fix any warnings or errors
- [x] 2.2 Run `cargo test` and fix any failures
- [x] 2.3 Run `git add -A && nix build .` and fix any build errors
