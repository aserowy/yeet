## Context

`open::selected` checks `find_nearest_directory_in_sibling(window).is_some()` and returns empty if false. The `split` command in `split.rs` shows the pattern for creating a new directory window: allocate empty buffers with `app::get_empty_buffer`, create a `Window::create(...)` directory, replace the current window with a horizontal split, then emit `NavigateToPath` to load content.

## Goals / Non-Goals

**Goals:**
- Pressing Enter in copen with no sibling directory creates a horizontal split (directory above, copen below), focuses the directory, and navigates to the selected path.

**Non-Goals:**
- Changing behavior when a sibling directory already exists (that path stays the same).

## Decisions

**Extend the `else` branch in `open::selected` to create a split on-demand.**

When `find_nearest_directory_in_sibling` returns `None`:
1. Update `qfix.current_index` to the cursor position and refresh the copen buffer.
2. Allocate 3 empty buffers for the new directory window (`app::get_empty_buffer`).
3. Replace the current window tree: `mem::take` the current window (which contains QuickFix), create `Window::Horizontal { first: Window::create(...), second: old_window, focus: SplitFocus::First }`.
4. Emit `NavigateToPathAsPreview(path)` — since focus is now on the directory window, `navigate_to_path_with_selection` will find directory viewports and succeed.

This follows the same `mem::take` + wrap pattern used by `qfix_window::open` and `split::create_split`.

## Risks / Trade-offs

[New window creation] Creates a directory window with empty buffers that get populated by the subsequent `NavigateToPathAsPreview` action. This is the same approach `split` uses. → No mitigation needed.
