## Context

`yeet-buffer` is a low-level buffer rendering crate. It currently defines `BufferTheme` with a `Default` impl containing hardcoded ANSI escape codes, and exposes two public view functions: `view()` (uses defaults) and `view_themed()` (accepts a theme). The centralized `Theme` in `yeet-frontend` already converts itself to a `BufferTheme` via `to_buffer_theme()`, and all call sites already use `view_themed`. The `view()` convenience function and `Default` impl are dead code paths that bypass the theme system.

## Goals / Non-Goals

**Goals:**
- Eliminate the `Default` impl on `BufferTheme` so the struct has no implicit color knowledge.
- Remove the `view()` convenience wrapper; rename `view_themed()` to `view()` as the single entry point.
- Ensure `yeet-frontend::Theme::to_buffer_theme()` remains the sole construction site for `BufferTheme`.

**Non-Goals:**
- Changing the `BufferTheme` struct fields or the ANSI rendering pipeline.
- Modifying the `Theme` token system or Lua configuration surface.
- Changing how `view::view` internally consumes the theme.

## Decisions

### Remove `Default` impl rather than deprecate

Remove the `Default` impl entirely instead of marking it `#[deprecated]`. Rationale: no external consumers exist outside this workspace, and a compile error is a clearer signal than a warning. The default values already live in `Theme::to_buffer_theme()`.

### Rename `view_themed` → `view` rather than keeping both

Since the only difference between the two functions is whether a default theme is injected, and we are removing the default, there is no reason to keep a separate name. Renaming avoids a confusing `_themed` suffix when theming is now mandatory. The old `view()` signature disappears entirely.

### Keep `BufferTheme` as a plain struct with public fields

No need to introduce a builder or constructor. The struct is constructed in exactly one place (`to_buffer_theme`), and public fields keep it simple for tests.

## Risks / Trade-offs

- **[Compile breakage for out-of-tree consumers]** → Acceptable: this is a workspace-internal crate with no published consumers. The breakage is intentional and caught at compile time.
- **[Test code using `BufferTheme::default()`]** → Any tests in `yeet-buffer` that rely on the default will need a helper or explicit construction. Mitigation: provide a `#[cfg(test)]` helper in `yeet-buffer` tests if needed.
