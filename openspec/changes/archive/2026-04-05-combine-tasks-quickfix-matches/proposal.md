## Why

Several match statements in `model/mod.rs` have separate `Window::QuickFix` and `Window::Tasks` arms with identical bodies. These can be combined using `|` patterns to reduce duplication.

## What Changes

- Combine 4 match arms in `yeet-frontend/src/model/mod.rs` where `Window::Tasks(vp)` and `Window::QuickFix(vp)` have the same body.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

_None. This is a pure refactoring with no behavior change._

## Impact

- `yeet-frontend/src/model/mod.rs` — 4 match arms combined
