## Why

Yeet's viewport settings (line numbers, wrap, borders, sign columns, cursor visibility) are hardcoded per window type. Users cannot customize how viewports look or behave based on context (e.g., hide line numbers in preview, enable wrap for certain directories). A Lua callback on window creation would let users inspect window context and override viewport settings, establishing the foundational pattern for all future Lua-driven customization hooks in yeet.

## What Changes

- Introduce a Lua callback system on the `y` global table that yeet invokes at defined lifecycle points, starting with window creation.
- Expose a Lua-accessible window context object containing viewport settings (line numbers, wrap, borders, sign column, cursor visibility) and window metadata (window type, path).
- Mutations to the context object's viewport settings are read back into Rust and applied to the created viewports.
- All hooks live under `y.hook.<hook_name>`. The callback mechanism is designed as a general-purpose pattern: a named function under `y.hook` receives a context table, the user modifies it, and yeet reads back the changes. Future hooks (e.g., `y.hook.on_navigate`, `y.hook.on_mode_change`) follow the same convention.

## Capabilities

### New Capabilities

- `lua-callbacks`: The general-purpose callback invocation pattern — how yeet registers, invokes, and reads back results from named Lua functions under `y.hook`. Covers the `y.hook` table structure, callback lifecycle, error handling, the contract between Rust and Lua for context objects, and the specific `y.hook.on_window_create` callback (when it fires, context structure, writable viewport settings, mutation read-back).

### Modified Capabilities

- `lua-runtime`: The `y` table contract expands from a static configuration namespace (theme values) to also hosting a `y.hook` subtable for callback functions that yeet invokes.

## Impact

- **New `yeet-lua` crate**: All Lua logic (runtime init, theme loading, hook invocation, table↔Rust conversion) extracted into a dedicated workspace crate. Sole owner of the mlua dependency. Depends on `yeet-buffer` for `ViewPort`/`LineNumber` types.
- **yeet crate**: `lua.rs` removed — Lua initialization moves to `yeet-lua`. The `yeet` crate calls `yeet-lua` for init and passes the `Lua` instance into `yeet-frontend`.
- **yeet-frontend crate**: Depends on `yeet-lua`. Window creation sites (all 7 across Directory, Help, QuickFix, Tasks) call `yeet-lua` invocation functions to apply hook overrides before layout.
- **yeet-buffer crate** (`model/viewport.rs`): ViewPort fields are already public; no structural changes needed.
- **User configuration** (`init.lua`): Users gain a new API surface — defining `y.hook.on_window_create = function(ctx) ... end` to customize viewport settings per window.
