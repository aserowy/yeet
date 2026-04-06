## Context

The statusline has 3 buffer types (Help, Tasks, QuickFix) with identical rendering logic. Each has a focused variant (label bold + position indicator) and an unfocused variant (label only). The only differences are the label string and how to get the line count from the buffer.

## Goals / Non-Goals

**Goals:**

- Eliminate duplicated statusline rendering code for Help, Tasks, and QuickFix.
- Keep the filetree (Directory) statusline untouched — it has unique logic (permissions, changes, path display).

**Non-Goals:**

- Changing any visual behavior.
- Refactoring the filetree statusline.

## Decisions

**Extract two shared functions: `label_status` and `label_status_unfocused`**

The focused function takes `(label: &str, line_count: usize, viewport, frame, rect, theme)`. The unfocused function takes `(label: &str, frame, rect, theme)`. The `view` match arms extract the line count from each buffer variant and pass it along with the label.

This is the simplest extraction — no traits, no generics on buffer types, just passing the two values that differ.

## Risks / Trade-offs

(none — pure deduplication of identical logic)
