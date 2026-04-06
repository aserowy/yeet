## Context

Yeet embeds Lua 5.4 via mlua in the `yeet` crate (`lua.rs`). Currently, Lua is used only for static theme configuration: `init.lua` sets values on `y.theme`, then `load_theme()` reads them back and returns a `Theme` struct before the UI starts. The Lua runtime is dropped after initialization — it does not persist into the event loop.

Viewport settings (line numbers, wrap, borders, cursor visibility, sign columns) are hardcoded in `Window::create()` and the various window creation sites (splits, tabs, quickfix open). The `Settings` struct holds global `WindowSettings` per pane type (parent/current/preview) but these only cover `sign_column_width` today.

Window creation happens at multiple call sites across all window types:

- **Directory**: `App::default()`, `create_tab()`, `create_split()`, `open::selected()` — all call `Window::create(parent_id, current_id, preview_id)` which returns a `Window::Directory` with hardcoded viewport configuration.
- **Help**: `help::open()` — constructs a `Window::Help(help_viewport)` and wraps the current window in a horizontal split.
- **QuickFix**: `qfix::window::open()` — constructs a `Window::QuickFix(qfix_viewport)` and wraps the current window in a horizontal split.
- **Tasks**: `task::open()` — constructs a `Window::Tasks(task_viewport)` and wraps the current window in a horizontal split.

The Lua runtime lives in the `yeet` crate, but `Window`/`ViewPort` types live in `yeet-buffer`/`yeet-frontend`. The callback system must bridge this crate boundary. Currently mlua is only a dependency of the `yeet` crate.

## Goals / Non-Goals

**Goals:**

- Persist the Lua runtime across the application lifetime so callbacks can be invoked at any point during execution
- Establish `y.hook` as the namespace for all lifecycle callbacks, starting with `y.hook.on_window_create`
- Let users inspect window context (type, path) and modify viewport settings (line numbers, wrap, borders, sign column width, cursor visibility) per pane
- Design the callback pattern to be trivially extendable for future hooks without architectural changes
- Gracefully handle missing hooks, Lua errors, and invalid values — never crash, always fall back to defaults

**Non-Goals:**

- Async Lua execution or coroutine support — callbacks are synchronous and blocking
- Exposing buffer content or cursor state to `on_window_create` — the window has no content yet at creation time
- Allowing Lua to create or destroy windows — only viewport settings on the window being created are writable
- Modifying the theme from hooks — theme remains a static `y.theme` configuration
- Hot-reloading `init.lua` during runtime

## Decisions

### 1. Persist the Lua runtime in Model

**Decision:** Keep the `Lua` instance alive for the entire application lifetime by storing it in `Model`.

**Rationale:** The current approach creates a `Lua` instance in `load_theme()`, reads theme values, and drops it. For callbacks to work, the runtime must persist so user-defined functions in `init.lua` remain callable. Storing it in `Model` keeps it accessible from all update paths where window creation occurs.

**Alternatives considered:**
- *Re-create Lua and re-execute init.lua on each callback*: Wasteful, stateless (user's runtime state lost), and slow.
- *Store in a global static*: mlua's `Lua` is `!Send` by default; a global static would require unsafe or feature flags. Passing through `Model` follows existing patterns.

### 2. Callback namespace: `y.hook`

**Decision:** All hooks are registered as functions on `y.hook.<name>`. For this change: `y.hook.on_window_create`.

**Rationale:** Separates callback functions from static config (`y.theme`). The `y.hook` table is a natural namespace that users can populate in `init.lua`. Yeet checks for the presence of each hook function before invocation — if `y.hook.on_window_create` is nil, no callback fires.

**User-facing API:**

```lua
y.hook.on_window_create = function(ctx)
  if ctx.type == "directory" then
    ctx.current.line_number = "absolute"
    ctx.current.wrap = true
    ctx.preview.hide_cursor_line = false
  end
end
```

### 3. Context table structure

**Decision:** Pass a single Lua table to the callback with window metadata and per-pane viewport settings. The table structure mirrors the window type.

For `Window::Directory`:

```lua
{
  type = "directory",     -- "directory", "help", "quickfix", "tasks"
  path = "/home/...",     -- target path (if known at creation time)
  parent = { ... },       -- viewport settings
  current = { ... },      -- viewport settings
  preview = { ... },      -- viewport settings
}
```

For single-viewport windows (`help`, `quickfix`, `tasks`):

```lua
{
  type = "help",
  viewport = { ... },     -- single viewport settings
}
```

Each viewport settings table contains:

```lua
{
  line_number = "none",          -- "none", "absolute", "relative"
  line_number_width = 0,
  sign_column_width = 0,
  show_border = false,
  hide_cursor = false,
  hide_cursor_line = false,
  wrap = false,
}
```

**Rationale:** Mirrors the Rust `ViewPort` struct fields that are meaningful for user customization. Excludes layout fields (x, y, width, height) and runtime state (buffer_id, cursor, indices) that users must not modify. The `path` field lets users apply settings conditionally.

**Alternatives considered:**
- *Single flat table*: Doesn't work for Directory windows with 3 panes.
- *UserData with methods*: More type-safe but heavier to implement and less natural to use in Lua. Plain tables are idiomatic Lua and trivially inspectable.

### 4. Read-back strategy: read the table after callback returns

**Decision:** After calling the Lua function, read the context table's viewport fields back into Rust and apply them to the `ViewPort` structs. Unknown keys are ignored. Invalid values (wrong type, unrecognized enum string) are ignored with a logged warning.

**Rationale:** This is simpler and more robust than UserData with metamethods. The callback runs, the table is read, done. No ongoing Lua-Rust coupling after the callback returns. This pattern generalizes trivially to future hooks — each hook defines its own context table shape and read-back logic.

### 5. Invocation point: after `Window::create`, before layout

**Decision:** Invoke the callback immediately after `Window::create()` returns and before the window enters the tree and gets layout dimensions assigned. Introduce a function (e.g., `invoke_on_window_create`) that takes the `Lua` instance and `&mut Window`, constructs the context table from the window's viewports, calls the hook, and writes back any changes.

**Rationale:** At this point the viewports exist with their defaults but haven't been sized yet. This is the natural place to override settings before they affect rendering. Calling after layout would require a re-layout.

**Call sites to instrument (all window types):**
- `App::default()` — initial Directory window on startup
- `create_tab()` — new tab (Directory)
- `create_split()` — horizontal/vertical split (Directory)
- `open::selected()` — split from quickfix into Directory
- `help::open()` — Help window creation
- `qfix::window::open()` — QuickFix window creation
- `task::open()` — Tasks window creation

### 6. Introduce a `yeet-lua` crate

**Decision:** Extract all Lua logic into a new `yeet-lua` workspace crate. This crate is the sole owner of the mlua dependency and provides a Rust API for initialization and hook invocation.

**Responsibilities of `yeet-lua`:**
- Lua runtime initialization (create `Lua` instance, set up `y` table with `y.theme` and `y.hook` subtables, execute `init.lua`)
- Theme loading (currently in `yeet/src/lua.rs` — moved here)
- Hook invocation API (e.g., `invoke_on_window_create(&Lua, &mut Window)`)
- All Lua table ↔ Rust type conversions (`ViewPort` fields to/from Lua tables, `LineNumber` enum to/from string)

**Dependency graph:**
- `yeet-lua` depends on `yeet-buffer` (for `ViewPort`, `LineNumber`), `yeet-frontend` (for `Window`, `Theme`), and `mlua`
- `yeet` depends on `yeet-lua` (for init) and `yeet-frontend` (for running the app)
- `yeet-frontend` does NOT depend on `yeet-lua` or `mlua` — it only receives the `Lua` instance opaquely via `Model`

**Rationale:** Centralizing all Lua logic in one crate avoids spreading mlua across `yeet` and `yeet-frontend`. It creates a clean boundary: `yeet-lua` owns the Lua↔Rust bridge, other crates never touch mlua directly. Future Lua features (new hooks, new config namespaces) are added in one place. The existing theme loading code in `yeet/src/lua.rs` moves here unchanged.

**`Model` stores `Option<Lua>`:** The `Lua` type from mlua is re-exported by `yeet-lua` or stored as an opaque handle. `yeet-frontend` passes it through to `yeet-lua` invocation functions without importing mlua itself. Since `yeet-lua` depends on `yeet-frontend` (for `Window`), `yeet-frontend` cannot depend on `yeet-lua` (circular). Instead, `yeet-frontend` stores the Lua instance as an opaque type and the `yeet` crate orchestrates passing it to `yeet-lua` invocation functions at the appropriate call sites.

**Alternative approach — invert the dependency:** `yeet-lua` depends on `yeet-buffer` only (for `ViewPort`, `LineNumber`). It exposes a function like `invoke_on_window_create(&Lua, window_type: &str, path: Option<&Path>, viewports: &mut [&mut ViewPort])`. The `yeet-frontend` crate depends on `yeet-lua` and calls this function at each window creation site, extracting viewports from the `Window` enum itself. This avoids `yeet-lua` depending on `yeet-frontend` and keeps the dependency graph acyclic: `yeet-buffer` ← `yeet-lua` ← `yeet-frontend` ← `yeet`.

**Chosen approach:** The inverted dependency — `yeet-lua` depends only on `yeet-buffer`, exposes viewport-level APIs, and `yeet-frontend` depends on `yeet-lua`. This keeps the dependency graph clean and acyclic.

**Extensibility pattern for future hooks:** Each hook defines a typed context struct in `yeet-lua` (e.g., `WindowCreateContext`, `StatuslineContext`). These structs contain only primitive types and `yeet-buffer` types — never `yeet-frontend` types. `yeet-frontend` constructs the context from its model, passes it to `yeet-lua`, and `yeet-lua` handles all Lua table conversion internally. This keeps mlua fully encapsulated in `yeet-lua` while allowing the function signatures to stay clean as hooks grow in complexity.

**Alternatives considered:**
- *Keep mlua in `yeet` crate and add to `yeet-frontend`*: Spreads Lua concerns across two crates. Both need to understand Lua table structures. Harder to maintain as more hooks are added.
- *Trait-based abstraction without separate crate*: Adds indirection without the organizational benefit of crate-level separation.

### 7. `y.hook` table initialization

**Decision:** During Lua init, create the `y.hook` table as an empty table before executing `init.lua`. After execution, the table may contain user-defined functions. The runtime reads from `y.hook` at each invocation point.

**Rationale:** Pre-creating the table lets users write `y.hook.on_window_create = function(ctx) ... end` directly. If the table didn't exist, they'd need to create it first.

## Risks / Trade-offs

**Synchronous blocking callbacks** → Callback execution blocks the event loop. For `on_window_create` this is fine (runs rarely, should be fast). Future hooks on hot paths (e.g., per-keystroke) would need profiling. Mitigation: document that hooks should be lightweight; add a timeout or warning if a callback exceeds a threshold in a later change.

**No type safety in Lua tables** → Users can set nonsense values (e.g., `line_number = 42`). Mitigation: validate and ignore invalid values with warnings logged. Always start from the existing defaults, only override fields that are present and valid.

**New `yeet-lua` crate** → One more crate to maintain in the workspace. Mitigation: the workspace already has 4 crates and the boundary is natural. The crate is focused (all Lua, nothing else) and avoids the worse outcome of spreading mlua across multiple crates.

**Lua instance is `!Send`** → Cannot move across threads. Mitigation: all window creation is on the main thread in the update loop, so this is not an issue for the current architecture.
