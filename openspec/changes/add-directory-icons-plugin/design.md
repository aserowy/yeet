## Context

Directory buffers currently present line numbers/signs and filename text, but there is no icon affordance for fast file-type recognition. The requested change introduces a hook-driven icon system where the core provides new hooks — integrated into the existing `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling — with full bufferline and window context, and an external plugin (`yeet-directory-icons`) owns all icon identification and text color logic. The plugin is loaded through the existing user plugin configuration path.

This touches multiple surfaces at once: plugin sourcing/loading, new hook infrastructure in the enumeration/path message handlers, buffer prefix rendering, cursor/edit-column semantics, and theme token registration. The implementation must preserve existing editing behavior (cursor and text operations remain filename-scoped) while introducing a visually distinct icon column whose content is entirely determined by the plugin via hooks.

## Goals / Non-Goals

**Goals:**
- Support `yeet-directory-icons` via the existing user plugin loading path, without requiring plugin-manager workflow changes.
- Render an icon column between line numbers and filenames in directory buffers.
- Make icon-column support a first-class prefix segment in `@yeet-buffer` so all buffer definitions can represent it.
- Implement icon-column drawing in shared `@yeet-buffer` prefix logic so the directory window (three `@yeet-buffer` instances) uses the same rendering path.
- Ensure the icon column is UI prefix content only and is not part of underlying editable buffer text.
- Use icon-column length default `0`; when `yeet-directory-icons` is loaded/executed it sets length to `1` via `on_window_create` hook.
- Introduce new hooks in the existing `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling that pass the complete bufferline and the given window (with all metadata) to the plugin. The plugin directly mutates each bufferline: it adds/replaces the icon in the icon column and colors the text. The hooks fire at the same point where directory content is set or updated, so the plugin processes entries as they arrive.
- All icon identification logic (glyph resolution by extension, filename, directory name) lives entirely in the plugin, not in the core.
- All text color/styling logic (for both icon glyph and filename text) lives entirely in the plugin, not in the core.
- Ensure icon column is non-editable and cursor starts/operates on filenames.
- Remove current built-in file/directory colorization path before enabling plugin-provided styling so there is a single source of truth.

**Non-Goals:**
- Reworking non-directory buffer rendering.
- Redesigning the full plugin manager UX or lock/sync/update commands.
- Adding a general-purpose arbitrary prefix-column framework.
- Placing any icon resolution or color mapping logic in the core; the core only provides hooks and the plugin mutates bufferlines directly.

## Decisions

1. Existing plugin loading integration
   - Decision: Load `yeet-directory-icons` through existing user plugin configuration and startup loading, with no plugin-manager behavior changes.
   - Rationale: Matches expected user workflow and avoids introducing unrelated plugin-manager scope.
   - Alternative considered: Vendoring as repository submodule. Rejected because user will install/configure plugin normally.

2. Mutation-based hook architecture: plugin directly edits bufferlines
   - Decision: The core introduces new hooks in the existing `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling that provide the **complete bufferline and the given window with all metadata** on each hook call. The hooks fire at the same point where directory content is being set or updated, so the plugin processes entries as they arrive during enumeration and filesystem change events. The plugin **directly mutates the bufferline**: it adds or replaces the icon in the icon column and colors the bufferline text. There is no request/response pattern — the plugin edits the bufferline in-place inside the hook handler. The core does not contain any icon resolution tables, extension mappings, or color rules.
   - Rationale: Reuses the existing message flow for directory content lifecycle. Direct mutation is simpler than request/response and gives the plugin full control over how it modifies each entry. Keeps the core generic and extensible; all domain-specific icon/color knowledge is encapsulated in the plugin.
   - Alternative considered: Request/response hook pattern where plugin returns icon + token. Rejected because mutation is more direct and avoids an intermediate data structure the core must interpret.

3. Icon column as buffer prefix segment
   - Decision: Extend shared `@yeet-buffer` line-prefix composition to support `[line number][icon column][content]` with configurable icon-column display width. The icon column content is populated by the plugin's direct mutation of the bufferline during hook execution.
   - Rationale: This keeps layout predictable, enables consistent buffer-level semantics, and allows other buffer types to adopt the same prefix contract.
   - Alternative considered: Prepending icon into filename text itself. Rejected because it complicates cursor/edit offsets and makes icon text editable.

4. Directory window integration uses shared buffer rendering
   - Decision: Treat the directory window as three `@yeet-buffer` instances and render icon prefixes through shared `@yeet-buffer` functions rather than window-local ad hoc drawing.
   - Rationale: Guarantees consistent behavior across all directory window sections and avoids duplicated rendering logic.
   - Alternative considered: Draw icons only in a directory window-specific layer. Rejected due to code duplication and drift risk.

5. Filename-anchored cursor semantics
   - Decision: Maintain logical cursor column origin at filename start; icon column is render-only and excluded from underlying buffer text.
   - Rationale: Prevents accidental edits to decorative metadata and preserves modal editing expectations.
   - Alternative considered: Allow cursor traversal over icon column. Rejected due to UX friction and unnecessary mode transitions.

6. Plugin-owned class mapping for icon + text styling
   - Decision: The plugin maintains one easy-to-extend class mapping list (files by extension/name, directories by name) that drives both icon and filename text styling, with seeded defaults for `.direnv`, `target`, `.git`, `.github`, and known file-icon rules. The core does not contain these mappings.
   - Rationale: Avoids duplicated configuration and keeps icon/text styling behavior consistent, all within the plugin.
   - Alternative considered: Shared mapping in core consumed by plugin. Rejected because it leaks domain knowledge into the core.

7. Nerd Font color defaults with overrideable tokens
   - Decision: The plugin seeds defaults from original Nerd Font icon colors, applies the mapped default color to both icon glyph and filename text for every entry matching a rule (extension/name/directory) by directly mutating bufferline styling. Token names are plugin-defined — the core does not standardize icon-color class names. Directories receive their own distinct icon token separate from the file default token. Theme/config overrides are resolved through existing token architecture.
   - Rationale: Provides recognizable out-of-the-box visuals while preserving full customization. Plugin-defined tokens keep the core decoupled from icon domain knowledge. Separate directory tokens enable independent visual treatment of file vs. directory entries.

8. Replace legacy directory-entry colorization
   - Decision: Remove existing built-in file/directory colorization in directory buffers and let the plugin's hook mutation be the sole source of directory-entry styling.
   - Rationale: Prevents duplicate/conflicting color logic and ensures predictable styling precedence.

9. Icon-column width defaults and hook activation
   - Decision: `@yeet-buffer` defaults icon-column width to `0`; `yeet-directory-icons` sets width to `1` from its `on_window_create` hook when loaded/executed.
   - Rationale: Ensures no visual/layout impact by default while enabling deterministic one-cell icon rendering at window creation time via plugin lifecycle hook.
   - Alternative considered: Always reserve one column. Rejected because it wastes space and changes layout even when icons are unavailable.

10. Deferred PathsAdded hooks fire on flush
   - Decision: When `PathsAdded` events are deferred during Insert mode, the per-bufferline hooks are also deferred. Hooks fire when deferred events are flushed (after leaving Insert mode), at the same point the path additions are processed.
   - Rationale: Keeps hook invocation coupled to content mutation timing. Avoids plugin mutations on bufferlines that haven't been added to the buffer yet.

## Risks / Trade-offs

- [Plugin unavailable/misconfigured in user setup] -> Mitigation: preserve width `0` fallback and emit clear runtime diagnostics; no icons or color changes without plugin.
- [Prefix width regressions in wrapped/narrow views] -> Mitigation: centralize width calculation and add viewport scenarios in specs/tests.
- [Hook performance on large directories] -> Mitigation: plugin can cache mapping lookups; hooks fire per-bufferline during existing `EnumerationChanged`/`EnumerationFinished`/`PathsAdded` processing, which already iterates entries, so no additional iteration overhead.
- [Hook timing/order issues] -> Mitigation: ensure `on_window_create` hook consistently applies width `1` before directory window content draw; per-bufferline hooks fire during the same message handling that sets content, so ordering is guaranteed.
- [Plugin mutating bufferlines incorrectly] -> Mitigation: core validates bufferline state after hook execution and falls back to empty icon column and default text color on invalid state.

## Migration Plan

1. Implement icon-column segment in `@yeet-buffer` shared rendering, including default width `0`.
2. Wire directory window (three `@yeet-buffer` instances) to use shared icon-column rendering path.
3. Define and implement new mutation hooks in `EnumerationChanged`, `EnumerationFinished`, and `PathsAdded` message handling that pass complete bufferline and window (with all metadata) to plugins. The plugin directly mutates each bufferline (sets icon, colors text).
4. Ensure deferred `PathsAdded` events (Insert mode) also defer hook invocation; hooks fire on flush when events are processed.
5. Integrate existing plugin loading to consume `yeet-directory-icons` and register its hook handlers.
6. Register/execute plugin `on_window_create` hook to set shared icon-column width to `1`.
7. Remove legacy built-in directory colorization path.
8. Update cursor mapping logic for non-editable icon column semantics.
9. Add documentation and tests for hook contract, mutation behavior, icon rendering, width defaults, hook behavior, token overrides, and cursor behavior.

Rollback strategy: remove icon-column rendering and hook infrastructure behind a focused revert; keep directory buffers rendering filenames without icons.
