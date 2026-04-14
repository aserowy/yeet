## Context

The project has accumulated explanatory comments that describe what adjacent code does. The project coding standards already mandate "never write comments in your code" — this change enforces that retroactively.

## Goals / Non-Goals

**Goals:**
- Remove all explanatory comments from production and test code
- Remove commented-out code blocks
- Establish clean baseline

**Non-Goals:**
- Removing TODO/NOTE/FIX/HACK/FIXME/SAFETY markers
- Removing section-header comments that organize large enums/structs in theme.rs
- Refactoring code for clarity (that's a separate concern)

## Decisions

### Remove in bulk, file by file

Process each file that contains explanatory comments, removing comment lines and any resulting blank line clusters. This is a mechanical change with no behavioral impact.

### Preserve TODO/NOTE markers

These serve as work-tracking markers and should remain until addressed.

## Risks / Trade-offs

- [Low risk] Purely cosmetic change, no behavioral impact. All tests must continue to pass.
