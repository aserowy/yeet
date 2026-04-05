## 1. Combine match arms in model/mod.rs

- [x] 1.1 Combine `Window::QuickFix(vp)` and `Window::Tasks(vp)` arms in `focused_viewport` (~line 172)
- [x] 1.2 Combine `Window::QuickFix(_)` and `Window::Tasks(_)` arms in `focused_window_mut` (~line 193)
- [x] 1.3 Combine `Window::QuickFix(vp)` and `Window::Tasks(vp)` arms in `focused_viewport_mut` (~line 200)
- [x] 1.4 Combine `Window::QuickFix(vp)` and `Window::Tasks(vp)` arms in `buffer_ids` (~line 219)

## 2. Verify

- [x] 2.1 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify no regressions
