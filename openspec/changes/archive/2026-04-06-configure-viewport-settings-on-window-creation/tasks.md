## 1. Create yeet-lua crate

- [x] 1.1 Scaffold `yeet-lua` crate: create `yeet-lua/` directory with `Cargo.toml` (depends on `yeet-buffer`, `mlua` with `lua54` and `vendored` features) and `src/lib.rs`
- [x] 1.2 Add `yeet-lua` to workspace `Cargo.toml` members
- [x] 1.3 Move theme loading logic from `yeet/src/lua.rs` into `yeet-lua/src/theme.rs` (resolve_config_path, run_init_lua, load_theme)
- [x] 1.4 Move existing lua tests from `yeet/src/lua.rs` into `yeet-lua` and verify they pass
- [x] 1.5 Update `yeet/Cargo.toml` to depend on `yeet-lua` instead of `mlua` directly
- [x] 1.6 Update `yeet/src/main.rs` to call `yeet_lua` for initialization instead of the local `lua` module
- [x] 1.7 Remove `yeet/src/lua.rs`
- [x] 1.8 Verify `cargo test`, `cargo clippy`, and `cargo fmt` pass across the workspace

## 2. Persist Lua runtime and initialize y.hook

- [x] 2.1 Change `yeet-lua` init to return both `Theme` and `Lua` instance (instead of just `Theme`)
- [x] 2.2 Create `y.hook` as an empty table during Lua initialization (alongside `y.theme`)
- [x] 2.3 Add `yeet-lua` as a dependency of `yeet-frontend`
- [x] 2.4 Store `Option<Lua>` in `yeet-frontend`'s `Model` struct (re-export `Lua` type from `yeet-lua`)
- [x] 2.5 Update `yeet/src/main.rs` to pass the `Lua` instance into `yeet_frontend::run` and store it in `Model`
- [x] 2.6 Write tests: `y.hook` table exists after init, user can assign functions to `y.hook`

## 3. Viewport settings ↔ Lua table conversion

- [x] 3.1 Implement `ViewPort` → Lua table conversion in `yeet-lua`: serialize `line_number`, `line_number_width`, `sign_column_width`, `show_border`, `hide_cursor`, `hide_cursor_line`, `wrap` into a Lua table
- [x] 3.2 Implement Lua table → `ViewPort` read-back in `yeet-lua`: parse each field with type validation, log warnings for invalid types or unrecognized enum strings, ignore unknown keys
- [x] 3.3 Implement `LineNumber` enum ↔ string conversion ("none", "absolute", "relative")
- [x] 3.4 Write tests: round-trip conversion, invalid type handling, unrecognized enum handling, unknown key ignoring

## 4. Hook invocation API

- [x] 4.1 Implement `invoke_on_window_create` in `yeet-lua` that accepts window type string, optional path, and mutable viewport references — builds context table, calls `y.hook.on_window_create`, reads back changes
- [x] 4.2 Handle Directory window context: build table with `type`, `path`, `parent`, `current`, `preview` viewport subtables
- [x] 4.3 Handle single-viewport window context (Help, QuickFix, Tasks): build table with `type` and `viewport` subtable
- [x] 4.4 Handle hook absence: if `y.hook.on_window_create` is nil, return immediately without error
- [x] 4.5 Handle non-function hook value: log warning and return without error
- [x] 4.6 Handle Lua errors: catch errors from hook invocation, log with stack trace, return without modifying viewports
- [x] 4.7 Write tests: hook modifies viewport fields, hook is nil (no-op), hook errors gracefully, hook with invalid values logs warning and preserves defaults

## 5. Instrument window creation call sites

- [x] 5.1 Instrument `App::default()` — invoke hook after initial Directory window creation (requires passing Lua to default or restructuring init)
- [x] 5.2 Instrument `create_tab()` — invoke hook after `Window::create`
- [x] 5.3 Instrument `create_split()` — invoke hook after `Window::create` for the new Directory window
- [x] 5.4 Instrument `open::selected()` — invoke hook after `Window::create` when splitting from quickfix
- [x] 5.5 Instrument `help::open()` — invoke hook after Help viewport construction, before wrapping in split
- [x] 5.6 Instrument `qfix::window::open()` — invoke hook after QuickFix viewport construction, before wrapping in split
- [x] 5.7 Instrument `task::open()` — invoke hook after Tasks viewport construction, before wrapping in split
- [x] 5.8 Write integration tests: verify hook fires for each window type with correct context structure

## 6. Documentation

- [x] 6.1 Add/update documentation in `docs/help/configuration.md` covering the `y.hook` namespace, `on_window_create` hook, context table structure, and viewport settings fields with examples
- [x] 6.2 Run `markdownlint` on all modified docs and fix any issues

## 7. Final validation

- [x] 7.1 Run `cargo test` across the full workspace
- [x] 7.2 Run `cargo clippy` and `cargo fmt` and fix any issues
