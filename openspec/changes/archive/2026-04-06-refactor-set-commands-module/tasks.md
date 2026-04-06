## 1. Create settings module

- [x] 1.1 Create `yeet-frontend/src/update/command/settings.rs` with a public `execute` function that takes `app`, `args`, `mode_before`, and `mode` and returns `Vec<Action>`
- [x] 1.2 Move wrap/nowrap argument parsing and `set_wrap` dispatch into the new `execute` function
- [x] 1.3 Handle empty and unknown `:set` arguments with error messages in the new module

## 2. Update command dispatcher

- [x] 2.1 Add `mod settings;` to `yeet-frontend/src/update/command/mod.rs`
- [x] 2.2 Replace the four `:set` match arms with a single `("set", args)` arm that delegates to `settings::execute`

## 3. Validation

- [x] 3.1 Run `cargo test`, `cargo clippy`, and `cargo fmt` and fix any issues
- [x] 3.2 Verify all existing `:set` command tests still pass without modification
