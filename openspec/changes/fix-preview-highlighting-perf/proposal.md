## Why

Before commit `c753b47` (directory icon plugin), syntax-highlighted file previews using syntect were extremely fast. After that commit and the subsequent security fixes (`79b2ecb` — SVG escape fix, `a0bb985` — truncation removal), two performance-impacting changes remain in the preview pipeline: (1) `strip_non_sgr_escape_sequences` is called on every highlighted line, and (2) the `on_bufferline_mutate` Lua hook fires for every preview line via `preview.rs`, invoking the Lua VM per line for content buffers. Neither of these existed before `c753b47`, and together they add significant overhead to every file preview — especially for files with many lines.

## What Changes

- **Keep `strip_non_sgr_escape_sequences` for the syntax path** — while syntect's `as_24_bit_terminal_escaped()` itself only emits SGR color codes (`\x1b[38;2;R;G;Bm`), the raw text content from the file passes through verbatim. A malicious or unusual file (e.g., one containing embedded `\x1b[2J` erase sequences) would have those escapes pass through syntect into the buffer pipeline, so sanitization is necessary for security. However, optimize the implementation to reduce per-line overhead (e.g., fast-path skip when no `\x1b` bytes are present).
- **Batch the `on_bufferline_mutate` hook** for content (preview) buffers — instead of invoking the Lua VM once per line, collect all lines and fire the hook in a single batch call. The hook is intentionally available for content buffers (plugins should be able to mutate preview lines), but the per-line Lua invocation overhead is the main performance bottleneck.
- Restore preview highlighting performance to pre-`c753b47` levels by optimizing these two hot paths.

## Capabilities

### New Capabilities

### Modified Capabilities

- `rendering/buffer`: The `on_bufferline_mutate` hook invocation for content (preview) buffers shall use a batch approach — firing the hook once for all lines rather than per-line — to reduce Lua VM overhead while preserving plugin extensibility.

## Impact

- **Code**: `yeet-frontend/src/task/sanitize.rs` — add fast-path optimization (skip character iteration when no `\x1b` is present in line)
- **Code**: `yeet-frontend/src/update/preview.rs` — batch `on_bufferline_mutate` hook invocation for content buffers
- **Code**: `yeet-lua/src/hook.rs` — add batch variant of `invoke_on_bufferline_mutate` or optimize the existing function
- **Performance**: Preview rendering for text files should return to pre-`c753b47` speed
- **Security**: Both chafa and syntect sanitization paths remain intact; the `strip_non_sgr_escape_sequences` call is kept to prevent file-embedded escape sequences from reaching the terminal
