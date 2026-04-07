## 1. Fix Test Conditions

- [x] 1.1 In `yeet-frontend/src/update/command/mod.rs`, change the condition in `split_no_args_from_tasks_falls_back_to_home` from `dirs::home_dir().is_some()` to `dirs::home_dir().filter(|p| p.exists()).is_some()`
- [x] 1.2 In `yeet-frontend/src/update/command/mod.rs`, change the condition in `vsplit_no_args_from_tasks_falls_back_to_home` from `dirs::home_dir().is_some()` to `dirs::home_dir().filter(|p| p.exists()).is_some()`

## 2. Verify

- [x] 2.1 Run `cargo test` to confirm all tests pass
- [x] 2.2 Run `cargo clippy` and `cargo fmt` to confirm no warnings or errors
- [x] 2.3 Run `nix build .` to confirm the build succeeds
