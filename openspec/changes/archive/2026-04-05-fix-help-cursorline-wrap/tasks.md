## 1. Fix BufferLine construction from highlighted content

- [x] 1.1 In `help::open` in `yeet-frontend/src/update/command/help.rs`, replace `.map(|l| BufferLine::from(l.as_str()))` with `.flat_map(|l| l.split_terminator('\n').map(BufferLine::from))` to split highlighted strings on newlines into separate buffer lines

## 2. Tests

- [x] 2.1 Add a test in `yeet-buffer/src/view/mod.rs` that verifies cursor line width equals viewport width when content contains a trailing newline inside ANSI escapes (reproducing the bug scenario)
- [x] 2.2 Add a test in `yeet-frontend/src/update/command/help.rs` verifying that `highlight_markdown` output split via `split_terminator` produces the expected number of `BufferLine`s (no extra empty lines)
- [x] 2.3 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify everything passes
