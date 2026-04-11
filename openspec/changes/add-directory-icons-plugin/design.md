## Context

Directory buffers currently present line numbers/signs and filename text, but there is no icon affordance for fast file-type recognition. The requested change introduces a hook-driven icon system where the core provides mutation hooks — integrated into all buffer population code paths — with full bufferline fields and buffer-type metadata, and an external plugin (`yeet-directory-icons`) owns all icon identification and text color logic. The plugin is loaded through the existing user plugin configuration path.

This touches multiple surfaces at once: plugin sourcing/loading, expanded hook infrastructure across all buffer types, buffer prefix rendering, cursor/edit-column semantics, trailing-slash naming convention for directories, removal of `ContentKind`, removal of `icon_style`, `BufferType` enum introduction, theme token interaction between plugins, plugin help documentation, runtime plugin help discovery in `:help`, and the design principle that all public-facing hook APIs use object-based metadata for extensibility.

## Goals / Non-Goals

**Goals:**
- Support `yeet-directory-icons` via the existing user plugin loading path, without requiring plugin-manager workflow changes.
- Render an icon column between line numbers and filenames in directory buffers.
- Make icon-column support a first-class prefix segment in `@yeet-buffer` so all buffer definitions can represent it.
- Implement shared `@yeet-buffer` icon-column rendering so the directory window (three `@yeet-buffer` instances) uses the same rendering path.
- Ensure the icon column is UI prefix content only and is not part of underlying editable buffer text.
- Use icon-column length default `0`; when `yeet-directory-icons` is loaded/executed it sets length to `1` via `on_window_create` hook.
- Expand `on_bufferline_mutate` to fire for all buffer types with full bufferline fields (prefix, content, search_char_position, signs, icon) and a read-only `buffer` metadata object (`ctx.buffer`) containing `type` (string) and `path` (string).
- Introduce a `BufferType` Rust enum for `invoke_on_bufferline_mutate` callers, replacing raw `&str` parameters. The enum maps to lowercase strings in Lua for type safety.
- The plugin checks the buffer type via `ctx.buffer.type` in each hook invocation and decides whether to act (e.g., only process `directory` type buffers).
- All public-facing hook APIs SHALL use object-based metadata patterns (e.g., `ctx.buffer`) to ensure extensibility — new metadata fields can be added without breaking existing plugin code or changing the mutable field surface.
- Remove the `icon_style` field from `BufferLine`. The plugin handles all content styling by mutating the `content` Ansi string directly. The core does not apply any icon-related styling.
- All icon identification logic (glyph resolution by extension, filename, directory name) lives entirely in the plugin.
- All text color/styling logic lives entirely in the plugin, applied by mutating the content Ansi string.
- Adopt trailing-slash naming convention: directory names in bufferline content always end with `/`.
- Remove `ContentKind` enum after trailing-slash adoption since directory-ness is encoded in the name itself.
- Remove `is_directory` parameter from the hook interface.
- Ensure icon column is non-editable and cursor starts/operates on filenames.
- Remove built-in file/directory colorization with no fallback; without the plugin, entries are plain unstyled text.
- Theme tokens set by the icons plugin can be overridden by theme plugins; the icons plugin respects existing theme values.
- Plugin-specific documentation (token references, usage guides) lives in each plugin's own `docs/help/` directory, not in core `docs/help/`.
- Extend `:help` command to discover and include help pages from plugin `docs/help/` directories at runtime, in addition to built-in compile-time pages. Core help pages take priority on name overlap.

**Non-Goals:**
- Reworking non-directory buffer rendering beyond adding hook invocation.
- Redesigning the full plugin manager UX or lock/sync/update commands.
- Adding a general-purpose arbitrary prefix-column framework.
- Placing any icon resolution or color mapping logic in the core.

## Decisions

1. Existing plugin loading integration
   - Decision: Load `yeet-directory-icons` through existing user plugin configuration and startup loading, with no plugin-manager behavior changes.
   - Rationale: Matches expected user workflow and avoids introducing unrelated plugin-manager scope.
   - Alternative considered: Vendoring as repository submodule. Rejected because user will install/configure plugin normally.

2. Hook fires for all buffer types with buffer metadata object
   - Decision: The `on_bufferline_mutate` hook fires for every buffer type (directory, content, help, quickfix, tasks) when bufferlines are created or updated. Each invocation provides a read-only `buffer` metadata object (`ctx.buffer`) containing `type` (string matching `BufferType` enum variant names) and optionally `path` (parent path for directory buffers, file path for content buffers; absent/nil for help, quickfix, tasks). The plugin checks `ctx.buffer.type` and decides whether to act.
   - Rationale: A universal hook allows any plugin to customize any buffer type. The `buffer` metadata object lets plugins filter efficiently. Using an object (rather than flat fields) ensures extensibility — future metadata (e.g., buffer ID, line index) can be added as new fields on `ctx.buffer` without breaking existing plugins. Making `path` optional (nil when no path exists) keeps the metadata truthful — buffer types without paths don't expose an empty string that could be misinterpreted.
   - Alternative considered: Flat `buffer_type` and `path` fields on the context table. Rejected because flat fields make it harder to add new metadata without potentially conflicting with mutable field names; an object namespace cleanly separates read-only metadata from mutable bufferline fields.

2a. BufferType enum for type-safe hook invocations
    - Decision: Introduce a `BufferType` Rust enum with variants `Directory`, `Content`, `Help`, `Quickfix`, `Tasks`. The `invoke_on_bufferline_mutate` function accepts `BufferType` instead of `&str`. The enum implements a method (e.g., `as_str()`) that maps each variant to its lowercase string representation for injection into the Lua hook context. All callers pass a typed enum variant instead of a raw string literal.
    - Rationale: Using an enum instead of `&str` makes buffer type passing type-safe at compile time. Typos like `"diretory"` or `"contentt"` become compile errors rather than silent runtime bugs. The enum centralizes the set of valid buffer types and makes it easy to add new types in the future.
    - Alternative considered: Continue using `&str`. Rejected because string-based dispatch is fragile and offers no compile-time guarantees about valid values.

3. Full bufferline mutation in hook context
   - Decision: The hook context exposes all bufferline fields for mutation: `prefix`, `content` (Ansi string), `search_char_position`, `signs`, and `icon`. Line numbers are excluded (they are viewport metadata, not bufferline data). The plugin directly mutates these fields in-place.
   - Rationale: Giving plugins full access to the bufferline makes the hook maximally useful. Restricting to icon-only mutation would require expanding the API for every future plugin need.
   - Alternative considered: Exposing only `icon` and `icon_style`. Rejected because it limits plugin capabilities and requires core styling logic.

4. Remove icon_style — plugin owns all content styling
   - Decision: Remove the `icon_style` field from `BufferLine`. The plugin applies foreground color by prepending ANSI escape sequences to the `content` Ansi string and by including color sequences in the `icon` string. The core renders content as-is with no icon-related styling logic.
   - Rationale: The plugin already controls the ANSI content; having a separate `icon_style` field that the core applies creates a split responsibility. With the plugin handling all styling, the core is simpler and the plugin has full control.
   - Alternative considered: Keep `icon_style` for the icon column and let the plugin handle content styling. Rejected because it maintains a partial core styling path that complicates the architecture.

5. Trailing-slash naming convention for directories
   - Decision: Directory entry names in bufferline content always end with a trailing slash (`/`). This applies in the enumeration task runner (where directory entries are produced) and in `PathsAdded` handling. The trailing slash is part of the content string.
   - Rationale: The trailing slash makes directory-ness visible to users (they can see `target/` vs `target`) and to plugins (they check for trailing slash without needing a separate flag). This is a common convention (ls -F, fd, etc.).
   - Alternative considered: Keep names bare and pass `is_directory` separately. Rejected because it creates a hidden metadata dependency and doesn't improve UX.

6. Remove ContentKind enum
   - Decision: After adopting the trailing-slash convention, remove `ContentKind` entirely. The `(ContentKind, String)` tuples in enumeration messages become plain `String` values. The `is_directory` parameter in `invoke_on_bufferline_mutate` is also removed.
   - Rationale: With directory-ness encoded in the name, a separate discriminant is redundant. Removing it simplifies the message types and hook interface.
   - Alternative considered: Keep `ContentKind` as internal metadata even with trailing slashes. Rejected because it would be dead code that could drift from the naming convention.

7. Icon column as buffer prefix segment
   - Decision: Extend shared `@yeet-buffer` line-prefix composition to support `[signs][line number][icon column][custom prefix][border][content]` with configurable icon-column display width. The icon column content is populated by the plugin's direct mutation of the bufferline `icon` field during hook execution.
   - Rationale: This keeps layout predictable, enables consistent buffer-level semantics, and allows other buffer types to adopt the same prefix contract.
   - Alternative considered: Prepending icon into filename text itself. Rejected because it complicates cursor/edit offsets and makes icon text editable.

8. Directory window integration uses shared buffer rendering
   - Decision: Treat the directory window as three `@yeet-buffer` instances and render icon prefixes through shared `@yeet-buffer` functions rather than window-local ad hoc drawing.
   - Rationale: Guarantees consistent behavior across all directory window sections and avoids duplicated rendering logic.

9. Filename-anchored cursor semantics
   - Decision: Maintain logical cursor column origin at filename start; icon column is render-only and excluded from underlying buffer text.
   - Rationale: Prevents accidental edits to decorative metadata and preserves modal editing expectations.

11. DirectoryIconsColor theme tokens for all color mappings
    - Decision: The plugin registers a `DirectoryIconsColor*` theme token for every unique color in its rule set during `setup()`. Each mapping entry (extension, filename, directory name) references a token name. During bufferline mutation, the plugin resolves colors by reading these tokens from `y.theme`. Default values are set during `setup()` only if the token is not already present. Token names follow the pattern `DirectoryIconsColor<Identifier>` (e.g., `DirectoryIconsColorRs`, `DirectoryIconsColorMakefile`, `DirectoryIconsColorDefaultDirectory`).
    - Rationale: Registering all colors as theme tokens makes every icon/text color user-overrideable via `y.theme`. Theme plugins (e.g., `yeet-bluloco-theme`) can override any or all tokens. The "set only if absent" pattern during `setup()` ensures theme plugins loaded before the icons plugin take priority.
    - Alternative considered: Using hardcoded hex values during mutation and only exposing `BufferFileFg`/`BufferDirectoryFg` as fallbacks. Rejected because it limits per-extension color customization.

12. Nerd Font color defaults with overrideable DirectoryIconsColor tokens
    - Decision: The plugin seeds defaults from original Nerd Font icon colors into `DirectoryIconsColor*` tokens. Directories receive their own distinct token (`DirectoryIconsColorDefaultDirectory`). Theme plugins can override these tokens; the directory-icons plugin checks for existing theme values and does not overwrite them. The `yeet-bluloco-theme` plugin provides override values for all `DirectoryIconsColor*` tokens.
    - Rationale: Provides recognizable out-of-the-box visuals while preserving full customization. Theme plugin priority ensures consistent theming across all plugins.

12. Remove legacy directory-entry colorization without fallback
    - Decision: Remove existing built-in file/directory colorization in directory buffers with no fallback. Without the plugin, directory entries are plain unstyled text.
    - Rationale: Clean separation — the core never colors directory entries. This avoids conflicting style sources and simplifies the rendering pipeline.
    - Alternative considered: Keep a default fallback in the core. Rejected because it reintroduces the split responsibility this change eliminates.

13. Deferred PathsAdded hooks fire on flush
    - Decision: When `PathsAdded` events are deferred during Insert mode, the per-bufferline hooks are also deferred. Hooks fire when deferred events are flushed (after leaving Insert mode).
    - Rationale: Keeps hook invocation coupled to content mutation timing.

14. Plugin-specific documentation lives in plugin repos
    - Decision: Plugin-specific documentation (e.g., `DirectoryIconsColor*` token reference, plugin usage, configuration) is maintained in each plugin's own `docs/help/` directory. Core `docs/help/` files only document core concepts and do not reference optional plugin-specific tokens or behavior. The `yeet-directory-icons` plugin creates `docs/help/directory-icons.md` with its full token reference. The `yeet-bluloco-theme` plugin creates `docs/help/bluloco-theme.md` with its theme override documentation.
    - Rationale: Separating documentation ownership ensures core docs remain independent of optional plugins. Plugin authors maintain their own documentation alongside their code. Users find plugin-specific docs via `:help <plugin-topic>`.
    - Alternative considered: Keep all documentation in core `docs/help/`. Rejected because it couples core to optional plugins and plugin docs would be missing when not installed.

15. Extend `:help` to discover plugin help pages at runtime
    - Decision: The `:help` command scans each loaded plugin's directory for `docs/help/*.md` files at runtime and registers them as additional help pages. These runtime-discovered pages are searchable by page name and heading just like built-in pages. When a topic matches both a core help page and a plugin help page, the core page takes priority (core pages are searched first). Plugin help pages are only available when the corresponding plugin is loaded.
    - Rationale: Plugins need a way to provide discoverable documentation without modifying core source code. Runtime discovery aligns with the plugin architecture — plugins are self-contained. Core priority ensures users always get the canonical core documentation for any overlapping topic.
    - Alternative considered: Require plugins to register help via a Lua hook. Rejected because it adds unnecessary complexity; file-based discovery is simpler and convention-driven.

## Risks / Trade-offs

- [Plugin unavailable/misconfigured in user setup] -> Mitigation: preserve width `0` fallback; no icons or color without plugin. Users see plain unstyled filenames.
- [Prefix width regressions in wrapped/narrow views] -> Mitigation: centralize width calculation and add viewport scenarios in specs/tests.
- [Hook performance on large directories] -> Mitigation: plugin can cache mapping lookups; hooks fire per-bufferline during existing content population, so no additional iteration overhead.
- [Hook fires for all buffer types increasing overhead] -> Mitigation: plugin checks buffer type early and returns immediately for non-directory types. The per-invocation cost of a type check is negligible.
- [Trailing-slash convention affects sorting] -> Mitigation: sort function operates on stripped content; trailing slash makes directories sort after same-named files, which is acceptable behavior.
- [Removing ContentKind breaks existing code] -> Mitigation: systematic removal with the trailing-slash convention providing the same information.
- [Theme plugin vs icons plugin token conflict] -> Mitigation: icons plugin checks for existing theme values before setting defaults; theme plugins always win.
- [Plugin mutating bufferlines incorrectly] -> Mitigation: core preserves pre-hook bufferline state on hook errors and continues rendering.
- [Plugin help page name collides with core page] -> Mitigation: core pages always take priority; plugin pages are only returned when no core match exists.

## Migration Plan

1. Add trailing slash to directory entry names in the enumeration task runner and `PathsAdded` handling.
2. Remove `ContentKind` enum and update all message types and consumers to use plain strings with trailing-slash convention.
3. Remove `is_directory` parameter from `invoke_on_bufferline_mutate` hook interface.
4. Remove `icon_style` field from `BufferLine` and all related core styling logic (prepend in `line.rs`, style in `prefix.rs`).
5. Restructure `on_bufferline_mutate` hook context to use a read-only `buffer` metadata object (`ctx.buffer` with `type` and optional `path` fields) instead of flat `buffer_type`/`path` fields. The `path` field is only set for buffer types with an associated path (directory, content); it is absent/nil for help, quickfix, tasks.
6. Make `path` optional in `invoke_on_bufferline_mutate` — pass `Option<&Path>` instead of `&Path`. Only set `ctx.buffer.path` when a path is provided.
7. Introduce `BufferType` enum and update `invoke_on_bufferline_mutate` to accept `BufferType` instead of `&str`. Update all callers.
8. Add hook invocation to all buffer population code paths (content buffer, help buffer, quickfix buffer, tasks buffer) in addition to existing directory buffer paths.
9. Remove legacy built-in directory colorization path (already done; verify no fallback remains).
10. Ensure icon column rendering in `prefix.rs` works without `icon_style` — render icon glyph as-is from the `icon` field.
11. Update `yeet-directory-icons` plugin to: check `ctx.buffer.type` for buffer type, use trailing-slash for directory detection, register `DirectoryIconsColor*` theme tokens during `setup()` (set only if absent), resolve colors from theme tokens during mutation, include ANSI color in icon string.
12. Update `yeet-bluloco-theme` plugin to set all `DirectoryIconsColor*` tokens as overrides.
13. Move plugin-specific documentation from core `docs/help/theme.md` and `docs/help/hooks.md` to plugin `docs/help/` directories.
14. Create `docs/help/directory-icons.md` in `yeet-directory-icons` plugin with token reference and usage.
15. Create `docs/help/bluloco-theme.md` in `yeet-bluloco-theme` plugin with theme override documentation.
16. Extend `:help` command to discover and include help pages from plugin `docs/help/` directories at runtime, with core pages taking priority.
17. Update cursor mapping logic for non-editable icon column semantics (already done; verify still correct).
18. Update core documentation for hook contract changes, trailing-slash convention, and theme interaction (core-only content).
19. Run full check suite: `cargo fmt`, `cargo clippy`, `cargo test`, `git add -A && nix build .`.

Rollback strategy: revert trailing-slash convention, restore `ContentKind`, restore `icon_style`, and narrow hooks back to directory-only; keep directory buffers rendering filenames without icons.
