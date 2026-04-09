## Context

Directory buffers currently present line numbers/signs and filename text, but there is no icon affordance for fast file-type recognition. The requested change introduces a plugin-owned icon system that is shipped in-repo as a submodule and rendered directly in the directory buffer layout.

This touches multiple surfaces at once: plugin sourcing/loading, buffer prefix rendering, cursor/edit-column semantics, and theme token registration. The implementation must preserve existing editing behavior (cursor and text operations remain filename-scoped) while introducing a visually distinct icon column.

## Goals / Non-Goals

**Goals:**
- Vendor `yeet-directory-icons` as a git submodule at `plugins/directory-icons` and make it available to runtime plugin loading.
- Keep plugin name/identity as `yeet-directory-icons` while using `plugins/directory-icons` as the in-repo folder name.
- Render an icon column between line numbers and filenames in directory buffers.
- Ensure the icon column is UI prefix content only and is not part of underlying editable buffer text.
- Resolve icon glyph and styling from filename extension, exact filename, and directory-name metadata.
- Ensure icon column is non-editable and cursor starts/operates on filenames.
- Support general styling for both icon glyph and filename text via shared class mapping with stable defaults.

**Non-Goals:**
- Reworking non-directory buffer rendering.
- Redesigning the full plugin manager UX or lock/sync/update commands.
- Adding a general-purpose arbitrary prefix-column framework.

## Decisions

1. Vendored plugin delivery
   - Decision: Add `git@github.com:aserowy/yeet-directory-icons.git` as a repository submodule at `plugins/directory-icons`.
   - Rationale: This gives deterministic source control, offline availability, and avoids first-run network dependency for icon support.
   - Alternative considered: Installing via existing plugin registry/update flow. Rejected because that path currently optimizes remote plugin lifecycle, not core-distribution plugins that must exist at startup.

2. Icon column as buffer prefix segment
   - Decision: Extend directory line prefix composition to include `[line number][icon column][filename]` where icon column has fixed display width.
   - Rationale: This keeps layout predictable and avoids filename jitter between entries.
   - Alternative considered: Prepending icon into filename text itself. Rejected because it complicates cursor/edit offsets and makes icon text editable.

3. Filename-anchored cursor semantics
   - Decision: Maintain logical cursor column origin at filename start; icon column is render-only and excluded from underlying buffer text.
   - Rationale: Prevents accidental edits to decorative metadata and preserves modal editing expectations.
   - Alternative considered: Allow cursor traversal over icon column. Rejected due to UX friction and unnecessary mode transitions.

4. Shared class mapping for icon + text styling
   - Decision: Use one easy-to-extend class mapping list (files by extension/name, directories by name) that drives both icon and filename text styling.
   - Rationale: Avoids duplicated configuration and keeps icon/text styling behavior consistent.
   - Alternative considered: Separate lists for icon classes and text classes. Rejected due to drift risk and maintenance overhead.

5. Nerd Font color defaults with overrideable tokens
   - Decision: Seed defaults from original Nerd Font icon colors, then expose theme/config overrides through existing token architecture.
   - Rationale: Provides recognizable out-of-the-box visuals while preserving full customization.
   - Alternative considered: Theme-only defaults unrelated to icon metadata. Rejected because it loses expected visual identity.

## Risks / Trade-offs

- [Submodule drift or detached updates] -> Mitigation: pin submodule commit and document update workflow.
- [Prefix width regressions in wrapped/narrow views] -> Mitigation: centralize width calculation and add viewport scenarios in specs/tests.
- [Token or mapping bloat from many file types] -> Mitigation: maintain a single mapping table with class fallback, and generate defaults only for known Nerd Font icons.
- [Plugin not present in shallow checkouts] -> Mitigation: startup diagnostics that clearly report missing vendored plugin path.

## Migration Plan

1. Add and commit the `plugins/directory-icons` submodule.
2. Integrate plugin loading path and expose icon lookup API to directory rendering.
3. Extend theme token registration/defaults for icon classes and aligned filename-text classes.
4. Update directory buffer rendering and cursor mapping logic for non-editable icon column.
5. Add documentation and tests for icon rendering, token overrides, and cursor behavior.

Rollback strategy: remove icon-column rendering and plugin hookup behind a focused revert; keep directory buffers rendering filenames without icons.

## Open Questions

- Which concrete icon-color token class names should be standardized (for example language, archive, media, config, default)?
- Should directories receive their own icon token separate from file default?
