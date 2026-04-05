## Context

The `resolve_topic` function in `help.rs` uses `==` for all string comparisons when matching topics against page names, page titles, section headings, and entry identifiers. This requires users to match exact casing.

## Goals / Non-Goals

**Goals:**

- Make all topic matching in `resolve_topic` case-insensitive.

**Non-Goals:**

- Fuzzy matching or partial matching of topics.
- Changing how topics are displayed in error messages.

## Decisions

**Use `eq_ignore_ascii_case` for all comparisons**

Replace all `==` comparisons in `resolve_topic` with `eq_ignore_ascii_case`. This covers page name lookup and all heading-level comparisons. ASCII case folding is sufficient since help content uses only ASCII identifiers.

Alternative considered: `to_lowercase()` on both sides. Rejected because `eq_ignore_ascii_case` avoids allocations and is the idiomatic Rust approach for ASCII content.

## Risks / Trade-offs

- [Ambiguous matches across casing] → Not a risk in practice. Help page names, section headings, and entry identifiers are unique regardless of casing. The first-match-wins behavior is preserved.
