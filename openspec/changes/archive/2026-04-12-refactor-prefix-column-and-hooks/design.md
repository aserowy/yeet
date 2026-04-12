## Context

The current yeet buffer model uses two separate concepts for line prefix content: a fixed-width `icon_column_width` on `ViewPort` with an `icon` field on `BufferLine`, and a per-line `prefix` field on `BufferLine` that has no fixed width allocation. The icon column was purpose-built for the `yeet-directory-icons` plugin, making the model unnecessarily specific. Additionally, Nerd Font icons typically occupy two terminal cells, but the current design assumes one cell.

Several other issues compound this: the `on_bufferline_mutate` hook fires before signs are set on bufferlines (meaning plugins cannot see or interact with signs); help page discovery for plugins is done at runtime in the frontend rather than at spec initialization; and core help docs contain plugin-specific guidance.

## Goals / Non-Goals

**Goals:**
- Unify icon rendering into the prefix column, removing the dedicated icon column concept
- Make prefix column width configurable per-viewport with sensible defaults (0 for most buffers, 1 for commandline, 2 for directory-icons plugin)
- Right-align prefix text within the allocated prefix column width
- Fix hook firing order so `on_bufferline_mutate` fires after signs are applied
- Move plugin help page discovery into `yeet-plugin` spec initialization (Rust side)
- Remove plugin-specific documentation from core help pages

**Non-Goals:**
- Changing the plugin loading or registration flow
- Adding new hook types or changing the hook API beyond field renaming
- Modifying the sign column or line number rendering
- Changing how core help pages are embedded at compile time

## Decisions

### Decision 1: Merge icon column into prefix column

**Choice:** Remove `icon_column_width` from `ViewPort` and `icon` from `BufferLine`. Replace with `prefix_column_width` on `ViewPort` that reserves fixed-width space for the existing `prefix` field.

**Rationale:** The icon column was a special-purpose allocation for a single plugin's use case. The `prefix` field already exists on `BufferLine` and is mutable via hooks. By giving it a fixed-width reservation on the viewport, plugins can use it for icons, status indicators, or any other per-line prefix content. This generalizes the model without losing functionality.

**Alternative considered:** Keep both `icon` and `prefix` as separate fields with separate width allocations. Rejected because it adds complexity without benefit — a single configurable prefix column serves all use cases.

### Decision 2: Default prefix column width is 0, configured per-viewport

**Choice:** `prefix_column_width` defaults to `0` on all viewports. The commandline buffer sets it to `1` (for command count display). The `yeet-directory-icons` plugin sets it to `2` via `on_window_create` hook.

**Rationale:** Most buffers don't need a prefix column. The commandline already uses `prefix` for count display and needs exactly 1 cell. Nerd Font icons are typically double-width glyphs, so the directory-icons plugin needs 2 cells. Per-viewport configuration via hooks maintains the existing plugin extension pattern.

**Alternative considered:** A global default of 2 for all buffers. Rejected because it wastes space on buffers that don't use prefixes (help, quickfix, tasks, content preview).

### Decision 3: Right-align prefix text within the column

**Choice:** When rendering the prefix column, right-align the prefix text within the allocated `prefix_column_width`. Pad with spaces on the left if the prefix content is narrower than the column width.

**Rationale:** Right alignment keeps icon glyphs visually adjacent to the content text rather than flush against the sign/line-number area. This produces cleaner visual output, especially when the prefix column is wider than the content (e.g., a 1-cell icon in a 2-cell column).

### Decision 4: Fix hook firing order — hooks fire after signs

**Choice:** In all code paths that invoke `on_bufferline_mutate` (enumeration, content setting, help, quickfix, tasks), move the hook invocation to fire after `set_sign_if_marked` and `set_sign_if_qfix` calls.

**Rationale:** Plugins need to see the complete bufferline state including signs. The current order in `update/enumeration.rs` fires hooks before signs, meaning plugins cannot react to sign state. The fix is a simple reordering of existing calls.

### Decision 5: Move plugin help page discovery to yeet-plugin spec initialization

**Choice:** Extend `PluginSpec` with a `help_pages` field (list of resolved help page paths). During spec initialization in `yeet-plugin` (Rust side), scan each plugin's `docs/help/` directory and populate the field. The frontend's `update/command/help.rs` reads from the spec instead of doing its own filesystem discovery.

**Rationale:** This moves filesystem I/O to initialization time rather than runtime. It also decouples help page discovery from the frontend, making it testable and reusable. The spec already knows each plugin's storage path, so it's the natural place for this resolution.

**Alternative considered:** Adding help pages to the Lua-side spec. Rejected because the requirement explicitly specifies Rust-side resolution, and it avoids Lua/Rust boundary overhead for filesystem operations.

### Decision 6: Remove plugin-specific content from core help docs

**Choice:** Edit `docs/help/hooks.md` to remove plugin-specific guidance like "Strip the trailing slash before performing filename-based icon resolution" and any other content that documents specific plugin behavior rather than core hook mechanics.

**Rationale:** Plugin-specific documentation belongs in the plugin's own `docs/help/` directory, as already established by the plugin help page spec. Core docs should only document the hook API surface, not prescribe plugin implementation patterns.

## Risks / Trade-offs

- **[Breaking change for existing plugins]** → Plugins using `icon` field or `icon_column_width` must migrate to `prefix`/`prefix_column_width`. Mitigation: the `yeet-directory-icons` plugin is the only known consumer and will be updated as part of this change.

- **[Prefix width assumes fixed-width characters]** → `prefix_column_width` is in terminal cells, but ANSI escape sequences in prefix content are zero-width. Mitigation: the existing prefix rendering already handles ANSI content via the `Ansi` type; width calculation uses `count_chars()` which strips escapes.

- **[Hook reordering may surface latent plugin bugs]** → Plugins that assumed hooks fire before signs might behave differently when signs are already present. Mitigation: the `yeet-directory-icons` plugin does not interact with signs, so this is low risk. The new order is strictly more correct.

- **[Help page discovery at spec init adds startup cost]** → Scanning plugin directories for help files happens during initialization. Mitigation: this is a small number of filesystem reads (one directory listing per plugin) that replaces equivalent runtime reads.
