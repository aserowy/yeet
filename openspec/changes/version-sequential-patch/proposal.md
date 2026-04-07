## Why

The current versioning scheme uses the day of the month as the patch component (`vYYYY.M.D`), with collision handling that increments the day beyond valid calendar dates. This produces confusing version numbers like `v2026.4.31` for April. Switching to a sequential patch number (`vYYYY.M.0`, `vYYYY.M.1`, ...) makes versions cleaner and unambiguous regardless of how many releases happen in a month.

## What Changes

- Change version tag format from `vYYYY.M.D` to `vYYYY.M.PATCH` where PATCH is a sequential number starting at 0
- The workflow finds the highest existing patch number for the current year-month and increments by one
- First release in a new month starts at patch 0
- Update `Cargo.toml` version and release documentation accordingly

## Capabilities

### New Capabilities

### Modified Capabilities

- `devops`: Version tag computation changes from day-based to sequential patch numbering per year-month

## Impact

- `.github/workflows/automated-release.yml`: Version computation logic changes
- Version tag format changes from `v2026.4.6` to `v2026.4.0` style — existing tags remain unchanged
- `docs/releasing.md`: Updated to reflect new versioning scheme
