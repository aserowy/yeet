## Context

The automated release workflow currently computes versions as `vYYYY.M.D` using the current date and increments the day on collision. This change replaces the day-based patch with a sequential number per year-month combination.

## Goals / Non-Goals

**Goals:**

- Replace day-based patch with sequential numbering starting at 0
- Determine the next patch by finding the highest existing tag for the current year-month
- Keep the same `vYYYY.M.PATCH` three-component format so existing workflows still trigger

**Non-Goals:**

- Migrating or re-tagging existing releases
- Changing any other part of the release workflow (auth, Cargo.toml update, tag cleanup)

## Decisions

### 1. Find highest patch via git tag listing and sorting

Use `git tag -l "vYEAR.MONTH.*"` to list all tags for the current year-month, extract the patch component, sort numerically, and take the highest. Next patch = highest + 1. If no tags match, start at 0.

**Alternative considered**: Counting tags instead of finding the max — rejected because deleted or out-of-order tags would produce incorrect results.

### 2. No leading zeros in any component

Consistent with the existing scheme: year, month, and patch all use plain integers without leading zeros.

## Risks / Trade-offs

- **Existing tags use day-based patch numbers** -> No migration needed. Old tags like `v2026.4.6` will be treated as patch 6, so the next sequential release in April 2026 will be patch 11 (one above the current highest, `v2026.4.10` from the previous scheme). This is acceptable — versions only need to be unique and increasing.
