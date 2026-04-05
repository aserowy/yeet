## Context

The `:split` and `:vsplit` commands resolve their target path via `get_current_path()`, which calls `get_focused_directory_buffer_ids()`. This function only succeeds for `Window::Directory` leaves — for `Window::Tasks` and `Window::QuickFix`, it returns `None`, causing the split to fail with "Preview path could not be resolved."

The keybindings `C-w C-s` and `C-w C-v` invoke `:split` and `:vsplit` with no arguments, so they hit the same failure path.

## Goals / Non-Goals

**Goals:**
- When focused on a non-directory window (Tasks, QuickFix) and no arguments are given, split/vsplit falls back to the home directory.
- When focused on a non-directory window and an absolute path or mark is given, use that path.
- When focused on a non-directory window and a relative path is given, error with a clear message (no base directory to resolve against).

**Non-Goals:**
- Changing the behavior when focused on a Directory window (existing behavior preserved).
- Adding new keybindings or commands.

## Decisions

### Decision 1: Introduce a fallback path function for split commands

When `get_current_path()` returns `None`, the split/vsplit handlers will attempt to resolve the target using a separate code path:

- **No arguments**: Use the home directory (`dirs::home_dir()` or `std::env::var("HOME")`).
- **Absolute path or mark**: Resolve via `expand_path` variant that doesn't require a source path — marks resolve independently, and absolute paths need no base directory.
- **Relative path**: Return an error explaining that relative paths require a directory context.

**Why not modify `get_current_path` to return home for non-directory windows?** That would change semantics for all callers of `get_current_path`, not just split. The fallback logic is specific to split behavior.

### Decision 2: Path classification before expansion

When `get_current_path()` is `None` and args are non-empty, classify the argument:
1. Starts with `'` → mark lookup (works without a source path since marks store absolute paths)
2. Starts with `/` → absolute path (works without a source path)
3. Otherwise → relative path → error

This avoids calling `expand_path` with a fabricated source path and makes the intent explicit.

### Decision 3: Shared logic between split and vsplit

Both command handlers have identical path resolution logic. The new fallback logic will follow the same pattern — duplicated in both match arms, consistent with the existing code structure.

## Risks / Trade-offs

- [Home directory unavailable] → Extremely unlikely on supported platforms. If `HOME` is unset, emit an error message "Split failed. Home directory could not be resolved." and do not create the split.
- [User expects relative paths to work from Tasks window] → Clear error message explains that relative paths need a directory context. This is the correct behavior since there is no meaningful base directory.
