---
phase: 02-animation-engine
plan: 01
subsystem: animation
tags: [keyframe, keyframe_derive, CanTween, tween, easing, interpolation, rust]

# Dependency graph
requires:
  - phase: 01-rendering-pipeline-and-primitives
    provides: Circle, Rect, Line, Text primitives with builder API and Color type

provides:
  - Easing enum with 4 variants (Linear, EaseIn, EaseOut, EaseInOut)
  - Tween<P: CanTween + Clone> struct with value_at(t_secs) clamped interpolation
  - CircleState, RectState, LineState, TextState with #[derive(CanTween)] and to_*() methods
  - eidos::Easing and eidos::Tween importable from crate root

affects: [02-02-render-loop, 02-03-animation-example, future animation plans]

# Tech tracking
tech-stack:
  added: [keyframe 1.1, keyframe_derive 1.0]
  patterns: [TDD red-green-refactor for Rust features, f64 state fields to avoid u8 arithmetic overflow]

key-files:
  created:
    - src/animation/mod.rs
    - src/animation/easing.rs
    - src/animation/tween.rs
  modified:
    - Cargo.toml
    - src/lib.rs
    - src/primitives/circle.rs
    - src/primitives/rect.rs
    - src/primitives/line.rs
    - src/primitives/text.rs

key-decisions:
  - "EaseInOut at exactly t=0.5 returns the same value as Linear (symmetric midpoint — mathematical property, not a bug); test covers t=0.25 quarter-point instead"
  - "LineState includes opacity field (Line struct has opacity; plan pseudocode omitted it — matched actual struct)"
  - "keyframe_derive worked cleanly with #[derive(CanTween)] on all four State structs with no issues"
  - "Color channels in State structs are f64 (0.0..=255.0) — no u8 arithmetic overflow possible during interpolation"

patterns-established:
  - "State structs hold f64 fields for all animatable properties; to_*() methods clamp and cast to u8 at rendering time"
  - "Tween::value_at(t_secs) takes absolute scene time; computes local_t = (t_secs - start_time) / duration clamped to [0.0, 1.0]"

requirements-completed: [ANIM-01]

# Metrics
duration: 5min
completed: 2026-02-25
---

# Phase 2 Plan 01: Animation Foundation Summary

**Easing enum, Tween<P> generic interpolator, and f64 State structs for all four primitives using keyframe/keyframe_derive crates**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-25T13:22:00Z
- **Completed:** 2026-02-25T13:27:00Z
- **Tasks:** 1 (single atomic implementation commit covering all steps)
- **Files modified:** 10

## Accomplishments

- Added `keyframe 1.1` and `keyframe_derive 1.0` to Cargo.toml
- Created `src/animation/` module with `Easing` enum and `Tween<P>` generic struct
- Added `CircleState`, `RectState`, `LineState`, `TextState` to all four primitive files with `#[derive(CanTween)]`
- Each State struct has a `to_*()` method that clamps and converts back to the concrete primitive type
- Exported `Easing` and `Tween` from crate root (`eidos::Easing`, `eidos::Tween`)
- 7 new tween unit tests covering: t=0 returns start, t=duration returns end, clamping below/above range, linear midpoint, easing non-linearity
- All 29 library tests pass; clean build with no warnings

## Task Commits

1. **Animation foundation (TDD green phase)** - `327f741` (feat)

**Plan metadata:** (pending final docs commit)

## Files Created/Modified

- `src/animation/easing.rs` - Easing enum with 4 variants
- `src/animation/tween.rs` - Tween<P: CanTween + Clone> with value_at() and 7 inline unit tests
- `src/animation/mod.rs` - Module re-exports for Easing and Tween
- `Cargo.toml` - keyframe 1.1 and keyframe_derive 1.0 added
- `src/lib.rs` - pub mod animation; pub use animation::{Easing, Tween};
- `src/primitives/circle.rs` - CircleState with CanTween derive and to_circle()
- `src/primitives/rect.rs` - RectState with CanTween derive and to_rect()
- `src/primitives/line.rs` - LineState with CanTween derive and to_line() (includes opacity field)
- `src/primitives/text.rs` - TextState with CanTween derive and to_text(content: &str)

## Actual Field Names in State Structs (for 02-03 reference)

| Struct | Fields |
|--------|--------|
| CircleState | cx, cy, r, fill_r, fill_g, fill_b, opacity |
| RectState | x, y, width, height, fill_r, fill_g, fill_b, opacity |
| LineState | x1, y1, x2, y2, stroke_r, stroke_g, stroke_b, stroke_width, opacity |
| TextState | x, y, font_size, fill_r, fill_g, fill_b, opacity |

## Decisions Made

- **EaseInOut midpoint test fix:** EaseInOut at exactly t=0.5 is mathematically equal to Linear (it's a symmetric function). The test was updated to use t=0.25 (quarter-point) where the difference is measurable.
- **LineState includes opacity:** The actual `Line` struct has an `opacity` field, but the plan pseudocode omitted it from `LineState`. Matched actual struct fields.
- **keyframe_derive compatibility:** `#[derive(CanTween)]` worked without issues on all four State structs. No procedural macro conflicts.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test fix: EaseInOut midpoint equals Linear midpoint**
- **Found during:** Task 1 (TDD RED phase verification)
- **Issue:** `tween_easeinout_midpoint_differs_from_linear` test failed — EaseInOut at t=0.5 mathematically equals Linear (symmetric function property)
- **Fix:** Changed test to verify at t=0.25 (quarter-point) where EaseInOut lags behind Linear (ease-in phase)
- **Files modified:** src/animation/tween.rs
- **Verification:** All 7 animation tests now pass
- **Committed in:** 327f741

**2. [Rule 2 - Missing Critical] LineState includes opacity**
- **Found during:** Task 1 (reading line.rs before appending)
- **Issue:** Plan pseudocode for `LineState` omitted `opacity` but the `Line` struct has an `opacity` field; `to_line()` would be unable to set opacity without it
- **Fix:** Added `pub opacity: f64` to `LineState` and `.opacity()` call in `to_line()`
- **Files modified:** src/primitives/line.rs
- **Verification:** Build passes; linestate-based line has opacity support
- **Committed in:** 327f741

---

**Total deviations:** 2 auto-fixed (1 bug in test, 1 missing field)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered

- keyframe_derive `#[derive(CanTween)]` requires all fields implement `CanTween` (which `f64` does) — choosing f64 for all state fields was essential for this to work cleanly.

## Next Phase Readiness

- `Tween<P>` is ready to be called from the render loop (02-02)
- State structs have the field names documented above for 02-03 example usage
- `eidos::Tween` and `eidos::Easing` importable from crate root

---
*Phase: 02-animation-engine*
*Completed: 2026-02-25*
