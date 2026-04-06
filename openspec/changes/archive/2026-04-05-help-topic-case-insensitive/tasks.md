## 1. Case-insensitive topic resolution

- [x] 1.1 In `resolve_topic` in `yeet-frontend/src/update/command/help.rs`, replace all `==` comparisons with `eq_ignore_ascii_case`: page name lookup, `#` title matching, `##` section matching, and `` ### `identifier` `` matching

## 2. Tests

- [x] 2.1 Write tests verifying case-insensitive matching for page names (e.g., `Commands` matches `commands`)
- [x] 2.2 Write tests verifying case-insensitive matching for section headings (e.g., `file operations` matches `File Operations`)
- [x] 2.3 Write tests verifying case-insensitive matching for entry identifiers (e.g., `Split` matches `split`)
- [x] 2.4 Run `cargo test`, `cargo clippy`, and `cargo fmt` to verify everything passes
