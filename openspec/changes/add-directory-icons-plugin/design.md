## Context

Directory buffers currently present line numbers/signs and filename text, but there is no icon affordance for fast file-type recognition. The requested change introduces a plugin-owned icon system loaded through the existing user plugin configuration path and rendered directly in the directory buffer layout.

This touches multiple surfaces at once: plugin sourcing/loading, buffer prefix rendering, cursor/edit-column semantics, and theme token registration. The implementation must preserve existing editing behavior (cursor and text operations remain filename-scoped) while introducing a visually distinct icon column.

## Goals / Non-Goals

**Goals:**
- Support `yeet-directory-icons` via the existing user plugin loading path, without requiring plugin-manager workflow changes.
- Render an icon column between line numbers and filenames in directory buffers.
- Make icon-column support a first-class prefix segment in `@yeet-buffer` so all buffer definitions can represent it.
- Implement icon-column drawing in shared `@yeet-buffer` prefix logic so the directory window (three `@yeet-buffer` instances) uses the same rendering path.
- Ensure the icon column is UI prefix content only and is not part of underlying editable buffer text.
- Use icon-column length default `0`; when `yeet-directory-icons` is loaded/executed it sets length to `1` via `on_window_create` hook.
- Resolve icon glyph and styling from filename extension, exact filename, and directory-name metadata.
- Ensure icon column is non-editable and cursor starts/operates on filenames.
- Support general styling for both icon glyph and filename text via shared class mapping with stable defaults.
- Seed the shared mapping with defaults for directories (`.direnv`, `target`, `.git`, `.github`) and known Nerd Font file icons.
- Remove current built-in file/directory colorization path before enabling plugin-provided styling so there is a single source of truth.

**Non-Goals:**
- Reworking non-directory buffer rendering.
- Redesigning the full plugin manager UX or lock/sync/update commands.
- Adding a general-purpose arbitrary prefix-column framework.

## Decisions

1. Existing plugin loading integration
   - Decision: Load `yeet-directory-icons` through existing user plugin configuration and startup loading, with no plugin-manager behavior changes.
   - Rationale: Matches expected user workflow and avoids introducing unrelated plugin-manager scope.
   - Alternative considered: Vendoring as repository submodule. Rejected because user will install/configure plugin normally.

2. Icon column as buffer prefix segment
   - Decision: Extend shared `@yeet-buffer` line-prefix composition to support `[line number][icon column][content]` with configurable icon-column display width; directory buffers map content to filename text and actively populate icons.
   - Rationale: This keeps layout predictable, enables consistent buffer-level semantics, and allows other buffer types to adopt the same prefix contract.
   - Alternative considered: Prepending icon into filename text itself. Rejected because it complicates cursor/edit offsets and makes icon text editable.

3. Directory window integration uses shared buffer rendering
   - Decision: Treat the directory window as three `@yeet-buffer` instances and render icon prefixes through shared `@yeet-buffer` functions rather than window-local ad hoc drawing.
   - Rationale: Guarantees consistent behavior across all directory window sections and avoids duplicated rendering logic.
   - Alternative considered: Draw icons only in a directory window-specific layer. Rejected due to code duplication and drift risk.

4. Filename-anchored cursor semantics
   - Decision: Maintain logical cursor column origin at filename start; icon column is render-only and excluded from underlying buffer text.
   - Rationale: Prevents accidental edits to decorative metadata and preserves modal editing expectations.
   - Alternative considered: Allow cursor traversal over icon column. Rejected due to UX friction and unnecessary mode transitions.

5. Shared class mapping for icon + text styling
   - Decision: Use one easy-to-extend class mapping list (files by extension/name, directories by name) that drives both icon and filename text styling, with seeded defaults for `.direnv`, `target`, `.git`, `.github`, and known file-icon rules.
   - Rationale: Avoids duplicated configuration and keeps icon/text styling behavior consistent.
   - Alternative considered: Separate lists for icon classes and text classes. Rejected due to drift risk and maintenance overhead.

6. Nerd Font color defaults with overrideable tokens
   - Decision: Seed defaults from original Nerd Font icon colors, apply the mapped default color to both icon glyph and filename text for every entry matching a rule (extension/name/directory), then expose theme/config overrides through existing token architecture.
   - Rationale: Provides recognizable out-of-the-box visuals while preserving full customization and consistent rule-based behavior across all matching entries.
   - Alternative considered: Theme-only defaults unrelated to icon metadata. Rejected because it loses expected visual identity.

7. Replace legacy directory-entry colorization
   - Decision: Remove existing built-in file/directory colorization in directory buffers and route all directory-entry styling through the directory-icons class mapping.
   - Rationale: Prevents duplicate/conflicting color logic and ensures predictable styling precedence.
   - Alternative considered: Layer plugin colors on top of legacy colorization. Rejected due to ambiguity and maintenance complexity.

8. Icon-column width defaults and hook activation
   - Decision: `@yeet-buffer` defaults icon-column width to `0`; `yeet-directory-icons` sets width to `1` from its `on_window_create` hook when loaded/executed.
   - Rationale: Ensures no visual/layout impact by default while enabling deterministic one-cell icon rendering at window creation time via plugin lifecycle hook.
   - Alternative considered: Always reserve one column. Rejected because it wastes space and changes layout even when icons are unavailable.

## Risks / Trade-offs

- [Plugin unavailable/misconfigured in user setup] -> Mitigation: preserve width `0` fallback and emit clear runtime diagnostics.
- [Prefix width regressions in wrapped/narrow views] -> Mitigation: centralize width calculation and add viewport scenarios in specs/tests.
- [Token or mapping bloat from many file types] -> Mitigation: maintain a single mapping table with class fallback, and generate defaults only for known Nerd Font icons.
- [Hook timing/order issues] -> Mitigation: ensure `on_window_create` hook consistently applies width `1` before directory window content draw.

## Migration Plan

1. Implement icon-column segment in `@yeet-buffer` shared rendering, including default width `0`.
2. Wire directory window (three `@yeet-buffer` instances) to use shared icon-column rendering path.
3. Integrate existing plugin loading to consume `yeet-directory-icons` and provide icon lookup data.
4. Register/execute plugin `on_window_create` hook to set shared icon-column width to `1`.
5. Introduce the unified mapping defaults (directory names + known file icon rules) and wire icon lookup through it.
6. Extend theme token registration/defaults for icon classes and aligned filename-text classes.
7. Update cursor mapping logic for non-editable icon column semantics.
8. Add documentation and tests for icon rendering, width defaults, hook behavior, token overrides, and cursor behavior.

Rollback strategy: remove icon-column rendering and plugin hookup behind a focused revert; keep directory buffers rendering filenames without icons.

## Open Questions

- Which concrete icon-color token class names should be standardized (for example language, archive, media, config, default)?
- Should directories receive their own icon token separate from file default?
