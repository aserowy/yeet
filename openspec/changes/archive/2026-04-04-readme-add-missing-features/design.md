## Context

The README.md serves as the primary user-facing documentation for yeet's keybindings and commands. Several features have been implemented without corresponding README updates.

## Goals / Non-Goals

**Goals:**
- Document all implemented but undocumented keybindings and commands in the existing README table format.

**Non-Goals:**
- Changing any code or behavior.
- Restructuring the README layout.

## Decisions

Add entries to existing tables in their logical position:
- `:copen` goes in the commands table near the other quickfix commands (`:cfirst`, `:cl`, etc.)
- `gg`, `G` go in the "navigation and normal mode" table near `j`, `k`
- `Enter` goes in the "navigation mode" table near `h`, `l`

## Risks / Trade-offs

None. Documentation-only change.
