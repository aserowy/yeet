## Context

The theme system in `yeet-frontend/src/theme.rs` defines token constants, default colors, and conversion helpers. The `BufferTheme` struct in `yeet-buffer/src/lib.rs` carries a subset of colors into the buffer rendering layer. Currently:

- There is no token for the buffer background — it always inherits the terminal default.
- Two tokens (`CommandLineFg`, `CommandLineBg`) are defined with defaults but never referenced outside `theme.rs`.

## Goals / Non-Goals

**Goals:**
- Add a `BufferBg` token so themes can control the buffer background color.
- Remove dead token definitions to keep the token set honest.

**Non-Goals:**
- Implementing a full commandline or diff theming system — those tokens can be re-introduced when actual consumers exist.
- Changing the default visual appearance (the new `BufferBg` defaults to `Color::Reset`).

## Decisions

### 1. `BUFFER_BG` token defaults to `Color::Reset`

Using `Color::Reset` preserves the current behaviour (terminal default background) when no override is configured. This is consistent with other "transparent" defaults like `SPLIT_BORDER_BG` and `DIRECTORY_BORDER_BG`.

Alternative considered: defaulting to `Color::Black`. Rejected because it would force a visible change on terminals with non-black backgrounds.

### 2. `BufferTheme` gains a `buffer_bg` field

The buffer rendering in `yeet-buffer` already receives a `BufferTheme`. Adding `buffer_bg` there keeps the pattern consistent and avoids passing an extra parameter.

### 4. ANSI resets must re-apply `buffer_bg`

Buffer line content is built as ANSI strings. Multiple places emit `\x1b[0m` (full SGR reset) to end styled regions: line numbers (`prefix.rs`), signs (`prefix.rs`), search highlights (`line.rs`), and cursor line styling (`line.rs`). A bare `\x1b[0m` clears all attributes including background, falling back to the terminal default — which breaks custom `BufferBg`.

The fix: every non-cursor line gets `buffer_bg` prepended as an ANSI bg escape so the line starts with the right background. All `\x1b[0m` resets within line content are replaced with `\x1b[0m` followed by the `buffer_bg` ANSI bg escape, so the background is restored after each reset. The cursor line already applies `cursor_line_bg` via the same mechanism, so only non-cursor lines and shared prefix code need this treatment.

Alternative considered: replacing `\x1b[0m` with targeted resets (e.g., `\x1b[39m` for fg-only reset). Rejected because it would require tracking which attributes are active, adding complexity with no benefit.

### 5. Remove `to_buffer_theme()` — enforce explicit border token selection

The current `to_buffer_theme()` silently defaults border colors to `SPLIT_BORDER_FG`/`SPLIT_BORDER_BG`. This hides which border tokens a call site intends to use and makes it easy to accidentally apply split border colors to directory panes (or vice versa).

The fix: delete `to_buffer_theme()` and require all call sites to use `to_buffer_theme_with_border(fg_token, bg_token)`. This makes the border token choice visible and explicit at every call site:
- `buffer.rs` directory pane branch → `to_buffer_theme_with_border(DIRECTORY_BORDER_FG, DIRECTORY_BORDER_BG)`
- `buffer.rs` split pane branch → `to_buffer_theme_with_border(SPLIT_BORDER_FG, SPLIT_BORDER_BG)`
- `commandline.rs` → `to_buffer_theme_with_border(SPLIT_BORDER_FG, SPLIT_BORDER_BG)`

Alternative considered: keeping `to_buffer_theme()` as an alias. Rejected because the implicit default is the root cause of the confusion.

### 6. Fix split border propagation through directory windows

When a directory window is inside a vertical split, the split sets `draw_borders: Some(true)` on the directory context. The `Window::Directory` branch then creates `dir_context` with `is_directory_pane: true` while preserving `draw_borders: Some(true)` — which forces ALL three directory panes to show borders with directory colors. The preview pane's forced border is actually the split separator and should use split colors.

The fix: in `Window::Directory`, don't propagate `draw_borders` to the parent and current panes (they already have their own `show_border` from viewport layout). For the preview pane, when `draw_borders` is `Some(true)`, use `is_directory_pane: false` so it gets split border colors for the split separator.

### 7. Wire sign tokens into sign generation

`generate_sign` in `update/sign.rs` hardcodes ANSI escape codes (`\x1b[1;95m`, `\x1b[1;96m`) instead of using the `SignQfix`/`SignMark` theme tokens and the existing `sign_qfix_style()`/`sign_mark_style()` methods.

The fix: thread `&Theme` through the sign creation functions (`generate_sign`, `set`, `set_sign_if_qfix`, `set_sign_if_marked`, `set_sign_for_paths`) and use `theme.sign_qfix_style()`/`theme.sign_mark_style()`. Callers that already have `&Theme` (enumeration.rs, path.rs) just pass it through. Callers that don't (mark.rs, qfix.rs, command/qfix.rs) gain a `&Theme` parameter.

### 3. Hard-delete unused tokens

The two unused commandline tokens are removed from the `tokens` module, the `Default` impl, and the config parser. If a user's config file contains a removed key, it is silently ignored (existing parser behaviour for unknown keys).

Alternative considered: deprecation warnings. Rejected — the tokens were never wired to anything, so no user could have observed a visual effect from setting them.

## Risks / Trade-offs

- [Risk] A user's config sets `CommandLineFg` and expects it to work → Mitigation: the token never had an effect, so nothing changes visually. Silently ignored by parser.
- [Risk] Future commandline theming will need new tokens → Mitigation: they can be added when actual consumers are implemented, with proper wiring from the start.
