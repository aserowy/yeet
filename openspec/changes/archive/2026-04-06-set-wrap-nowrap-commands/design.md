## Context

Word wrap rendering is fully implemented in the view layer (`yeet-buffer/src/view/wrap.rs`), including word-break algorithms, ANSI style carry-over, cursor positioning, and viewport scrolling. Each `ViewPort` already has a `wrap: bool` field that controls rendering behavior. However, there is no runtime mechanism to toggle this field - wrap state is only set at viewport creation time.

The command system in `yeet-frontend/src/update/command/mod.rs` dispatches commands via `split_once(' ')` pattern matching. There is no existing `:set` command infrastructure.

Directory windows contain three viewports (parent, current, preview). Other window types (QuickFix, Tasks, Help) contain a single viewport.

## Goals / Non-Goals

**Goals:**

- Add `:set wrap` and `:set nowrap` commands to toggle viewport wrap state at runtime
- For Directory windows, apply wrap state to all three viewports consistently
- For single-viewport windows, apply to that viewport

**Non-Goals:**

- General `:set` command framework for arbitrary options (keep it simple, just wrap/nowrap)
- Persisting wrap state across sessions or in Lua config
- Per-pane wrap configuration for Directory windows (e.g., wrap only preview)

## Decisions

**1. Direct command matching over generic `:set` parser**

Add `("set", "wrap")` and `("set", "nowrap")` as direct match arms in the command dispatcher rather than building a general-purpose `:set` option parser. This matches the existing command pattern and avoids over-engineering for a single option. If more `:set` options are needed later, the match arms can be refactored into a dedicated module.

Alternative: Build a generic `set` command parser with option registry. Rejected because there are no other `:set` options planned and the complexity is not justified.

**2. Add `set_wrap` method on Window enum**

Add a method on `Window` that sets wrap on all viewports within the window variant. For `Directory`, this sets all three viewports. For `QuickFix`/`Tasks`/`Help`, it sets the single viewport. For split windows, it sets wrap on the focused leaf window recursively.

Alternative: Only set wrap on the focused viewport via `focused_viewport_mut()`. Rejected because Directory windows should have consistent wrap state across all three panes per the requirements.

**3. No persistence**

Wrap state is ephemeral - it resets when the window is recreated. Persistence via settings or Lua config is a separate concern that can be added later without changing the command interface.

## Risks / Trade-offs

- [Limited scope] Only wrap/nowrap is supported via `:set`. If many options need runtime toggling, this approach doesn't scale. → Mitigation: Refactor to a generic parser when a second option is needed.
- [Directory consistency] Users cannot wrap only the preview pane independently. → Mitigation: This matches the stated requirement. Per-pane control can be added later with different syntax (e.g., `:set preview.wrap`).
