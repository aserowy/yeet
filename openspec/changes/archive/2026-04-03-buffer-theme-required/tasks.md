## 1. Remove defaults from yeet-buffer

- [x] 1.1 Remove the `Default` impl block for `BufferTheme` in `yeet-buffer/src/lib.rs`
- [x] 1.2 Remove the `view()` convenience function (non-themed) from `yeet-buffer/src/lib.rs`
- [x] 1.3 Rename `view_themed()` to `view()` in `yeet-buffer/src/lib.rs`

## 2. Update yeet-frontend call sites

- [x] 2.1 Update the import in `yeet-frontend/src/view/buffer.rs` from `view_themed as buffer_view` to `view as buffer_view`

## 3. Fix tests

- [x] 3.1 Update any tests in `yeet-buffer` that use `BufferTheme::default()` to construct the struct explicitly or use a test helper
- [x] 3.2 Run `cargo build --workspace` and `cargo test --workspace` to verify everything compiles and passes
