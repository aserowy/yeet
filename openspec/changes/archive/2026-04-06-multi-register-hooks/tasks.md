## 1. Hook object with :add() method

- [x] 1.1 Create a shared hook metatable in `yeet-lua/src/lib.rs` with an `add` method that validates the argument is a function and appends it to the table's array part
- [x] 1.2 Change `setup_and_execute` to create `y.hook.on_window_create` as a table with the hook metatable (instead of leaving `y.hook` as an empty table)
- [x] 1.3 Write tests: `:add()` appends functions, multiple `:add()` calls store in order, `:add()` with non-function logs warning and is ignored

## 2. Invocation iterates callback list

- [x] 2.1 Rewrite `try_invoke_on_window_create` in `hook.rs` to iterate the hook table's array entries (1, 2, 3, ...) instead of reading a single function
- [x] 2.2 Call each callback with the shared context table so mutations accumulate across callbacks
- [x] 2.3 Wrap each callback call in error handling — log error with callback index and continue to next
- [x] 2.4 Read back viewport settings from the context table only once, after all callbacks have run
- [x] 2.5 Write tests: multiple callbacks invoked in order, mutations visible across callbacks, single callback error doesn't block others, empty callback list is no-op

## 3. Update documentation

- [x] 3.1 Update `docs/help/hooks.md` to use `:add()` API in all examples and document multi-registration behavior
- [x] 3.2 Run `markdownlint` on modified docs and fix any issues

## 4. Validation

- [x] 4.1 Run `cargo test` across the full workspace
- [x] 4.2 Run `cargo clippy` and `cargo fmt` and fix any issues
