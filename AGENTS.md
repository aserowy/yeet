# AGENTS.md

This repository is a Rust workspace with these crates:
- `yeet` (binary)
- `yeet-frontend`
- `yeet-buffer`
- `yeet-keymap`

Use this file as the default playbook for agentic changes in this repo.

## General

- ask if you are uncertain

## Build, lint, test

### Prerequisites
- Linux CI installs system deps: `pkg-config`, `libchafa-dev`, `libglib2.0-dev`.
- Rust edition: 2021 (workspace).

### Build
- Workspace build: `cargo build`
- Build a single crate: `cargo build -p yeet-frontend`
- Build with verbose output: `cargo build --verbose`

### Format (lint)
- Check formatting (CI): `cargo fmt --check`
- Format locally: `cargo fmt`

### Clippy (lint)
- Workspace clippy: `cargo clippy --all-targets --all-features`
- Single crate clippy: `cargo clippy -p yeet-frontend --all-targets --all-features`
- Clippy with warnings as errors: `cargo clippy --all-targets --all-features -- -D warnings`

### Tests
- Run all tests: `cargo test`
- Run a single crate's tests: `cargo test -p yeet-keymap`
- Run one test by name (filter): `cargo test -p yeet-keymap add_and_resolve_key_normal_dd`
- Run a specific test target: `cargo test -p yeet-keymap --test lib_tests`
- Run tests with verbose output: `cargo test --verbose`
- Run one test in a specific target: `cargo test -p yeet-keymap --test lib_tests add_and_resolve_key_normal_dd`

## Code style guidelines

### General Rust style
- Prefer explicit, readable control flow; avoid cleverness.
- Keep functions short and focused; split complex logic into helpers.
- Favor immutable bindings; use `mut` only when needed.
- Use `#[derive(...)]` for common traits; implement custom `Debug` only when needed.
- Avoid `todo!()` in paths that can be triggered by users.
- Keep public APIs small; prefer `pub(crate)` for internal helpers.

### Imports and module layout
- Use grouped imports with brace syntax for related items.
- Standard library imports first, then external crates, then local crate modules.
- Keep module boundaries clear: `model/`, `update/`, `view/`, `task/`, `init/`.
- Prefer `use crate::...` for local modules; avoid deep relative paths.
- Keep `mod` declarations near the top of the file.

### Formatting
- Use `cargo fmt` (rustfmt defaults; see `rustfmt.toml`).
- Keep line lengths reasonable; rustfmt will wrap as needed.
- Prefer trailing commas in multi-line struct/enum/array blocks.
- Align match arms only when it improves readability; otherwise let rustfmt decide.

### Types and data modeling
- Use enums to model state transitions and messages (Elm-like architecture).
- Prefer `Option<T>` over sentinel values.
- Use `Result<T, AppError>` or crate-specific errors for fallible paths.
- Keep domain types in `model` modules; messages in `message` modules.
- Use newtypes for IDs and sizes when mixing units is likely.
- Prefer `PathBuf`/`Path` over `String` for filesystem paths.

### Naming conventions
- Modules: `snake_case`.
- Types: `PascalCase`.
- Functions/variables: `snake_case`.
- Constants: `SCREAMING_SNAKE_CASE`.
- Use verbs for actions: `update_*`, `load_*`, `navigate_*`, `execute_*`.
- Use `*_state` for state snapshots and `*_config` for settings.

### Error handling
- Use `thiserror::Error` for error enums (see `yeet-frontend/src/error.rs`).
- Prefer returning errors up the stack with `?` instead of `unwrap()`/`expect()`.
- Use `tracing` to log errors and important state transitions.
- Avoid panics in normal control flow; reserve for truly impossible states.
- Prefer domain-specific error variants over stringly-typed errors.
- Convert external errors with `#[from]` where it keeps call sites clean.

### Logging and tracing
- Use `tracing::{debug, info, warn, error, trace}` consistently.
- Add `#[tracing::instrument]` on complex functions handling state changes.
- Prefer structured logs over stringly-typed dumps where possible.
- Avoid logging sensitive file contents or clipboard data.

### Architecture conventions
- Follow the existing Elm-ish structure:
  - `model`: state/data types
  - `update`: message handling & state transitions
  - `view`: rendering
  - `task`: async/background work
- Keep message passing in `Message`/`KeymapMessage`/`BufferMessage` types.
- Avoid cross-crate cyclical dependencies (keymap uses buffer types, not vice-versa).
- Prefer `update::*` helpers over large match blocks in top-level update functions.
- Keep async/background work in `task` modules and return messages to update.

### Tests
- Use `#[test]` functions with clear, descriptive names.
- Prefer deterministic tests; avoid timing-sensitive behavior.
- Keep fixtures lightweight; reuse helper constructors where sensible.
- Use `assert_eq!` over `assert!` when comparing values.

### Performance and allocation
- Be mindful of cloning large buffers; use references where possible.
- Use `Vec`/`HashMap` with `collect()` only when needed.
- Avoid repeated allocations in tight loops; pre-allocate when size is known.

### Safety and lints
- `unsafe` is forbidden at the workspace level (`unsafe_code = "forbid"`).
- Respect existing `#[allow(...)]` scopes (e.g., for large enums).
- Prefer `#[allow]` with a short rationale in a comment when scope is narrow.

## Cursor/Copilot rules

No Cursor rules (`.cursor/rules/`, `.cursorrules`) or Copilot rules
(`.github/copilot-instructions.md`) were found in this repository.

## Repository notes

- Workspace root: `Cargo.toml` defines shared deps and lint policy.
- CI runs: `cargo fmt --check`, `cargo build`, `cargo test`.
- Main binary entrypoint: `yeet/src/main.rs`.
- Keymap tests live in `yeet-keymap/tests/lib_tests.rs`.
