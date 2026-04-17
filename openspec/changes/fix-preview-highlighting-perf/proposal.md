## Why

Before commit `c753b47` (directory icon plugin), syntax-highlighted file previews using syntect were extremely fast. After that commit and the subsequent security fixes (`79b2ecb` — SVG escape fix, `a0bb985` — truncation removal), two performance-impacting changes were added to the preview pipeline: (1) `strip_non_sgr_escape_sequences` is called on every highlighted line, and (2) the `on_bufferline_mutate` Lua hook fires for every preview line via `preview.rs`, invoking the Lua VM per line for content buffers. Neither of these existed before `c753b47`.

**Key finding**: The `assets/logo.svg` file contains **zero** escape characters or control characters — it's pure XML/base64 text. The `strip_non_sgr_escape_sequences` sanitization has nothing to strip for this file (or for any file that doesn't contain raw `\x1b` bytes). The original terminal escape/breakout bug for the SVG was **not** caused by escape characters in the file content — it was caused by `ansi_to_tui` struggling with the pathological 517K-character ANSI string that syntect produces when highlighting a 517K-character line. The `strip_non_sgr_escape_sequences` fix addresses a theoretical attack vector (files with embedded escape sequences) but does **not** address the actual `logo.svg` bug. The real fix for `logo.svg` must address the root cause: the 517K line flows through the entire pipeline untruncated — syntect highlighting, `Ansi::new()` wrapping, Lua hook invocation (copying 2MB+ strings into/out of Lua VM), and `ansi_to_tui` parsing.

## What Changes

- **Truncate lines before highlighting** — re-introduce a line-length limit, but apply it **only to lines that exceed a reasonable threshold** (e.g., 10K chars) rather than at viewport width. This prevents pathological syntect/ansi_to_tui behavior on mega-lines while preserving correct cross-line parser state for normal files (the bug from `a0bb985` was caused by truncating at viewport width, which broke syntect state). Lines beyond this threshold are inherently unhelpful to preview character-by-character.
- **Keep `strip_non_sgr_escape_sequences` as a security defense** — add a fast-path skip when no `\x1b` byte is present in the input (covers the vast majority of files including `logo.svg`), eliminating per-character iteration overhead for clean lines.
- **Batch the `on_bufferline_mutate` hook** for content (preview) buffers — the per-line Lua VM invocation is a major bottleneck, especially when it copies 517K+ strings into and out of Lua. Use a batch approach: fire the hook once for all lines rather than per-line.

## Capabilities

### New Capabilities

### Modified Capabilities

- `rendering/buffer`: The `on_bufferline_mutate` hook invocation for content (preview) buffers shall use a batch approach — firing the hook once for all lines rather than per-line — to reduce Lua VM overhead while preserving plugin extensibility.

## Impact

- **Code**: `yeet-frontend/src/task/syntax.rs` — add large-line truncation threshold (e.g., 10K chars) before highlighting to prevent pathological mega-line processing
- **Code**: `yeet-frontend/src/task/sanitize.rs` — add fast-path optimization (skip character iteration when no `\x1b` is present in input)
- **Code**: `yeet-frontend/src/update/preview.rs` — batch `on_bufferline_mutate` hook invocation for content buffers
- **Code**: `yeet-lua/src/hook.rs` — add batch variant of `invoke_on_bufferline_mutate` or optimize the existing function
- **Performance**: Preview rendering for text files should return to pre-`c753b47` speed; `logo.svg` specifically will no longer cause terminal breakout because the 517K line is truncated before entering the pipeline
- **Security**: Both chafa and syntect sanitization paths remain intact; `strip_non_sgr_escape_sequences` is kept with a fast-path for clean lines
