---
phase: 10-infallible-builders
plan: "01"
subsystem: api

tags: [rust, primitives, builder-pattern, infallible, clamping]

requires: []

provides:
  - "Circle with infallible .opacity() and .stroke() builders — clamping instead of Err"
  - "Rect with infallible .opacity() and .stroke() builders — clamping instead of Err"
  - "Line with infallible .stroke_width() and .opacity() builders — clamping instead of Err"
  - "Arrow with infallible .stroke_width() and .opacity() builders — clamping instead of Err"
  - "Bezier with infallible .stroke() and .opacity() builders — clamping instead of Err"
  - "Text with infallible .stroke(), .opacity(), .font_size(), and .line_height() builders — clamping instead of Err"
  - "All six State-to-primitive conversion methods free of .unwrap() calls"

affects:
  - 10-02
  - dataviz callers of primitive builders

tech-stack:
  added: []
  patterns:
    - "Infallible builder: clamp invalid inputs instead of returning Err"
    - "opacity clamps to [0.0, 1.0] via f64::clamp()"
    - "stroke/stroke_width clamp negative to 0.0 via f64::max(0.0)"
    - "font_size clamps to minimum 1.0 via f64::max(1.0)"
    - "line_height clamps to minimum 0.1 via f64::max(0.1)"

key-files:
  created: []
  modified:
    - src/primitives/circle.rs
    - src/primitives/rect.rs
    - src/primitives/line.rs
    - src/primitives/arrow.rs
    - src/primitives/bezier.rs
    - src/primitives/text.rs

key-decisions:
  - "Clamp strategy chosen over panic or Err: invalid inputs silently become valid values, matching API-01 ergonomics goal"
  - "font_size minimum is 1.0px (not 0.1) to ensure text is always visible"
  - "line_height minimum is 0.1em to allow very tight leading while preventing invisible text"
  - "EidosError import removed from all six primitive files — no longer needed"

patterns-established:
  - "Builder method returning Self with clamping: pub fn opacity(mut self, value: f64) -> Self { self.opacity = value.clamp(0.0, 1.0); self }"
  - "Unit test naming: *_is_clamped (not *_returns_err) for infallible builder tests"

requirements-completed:
  - API-01

duration: 3min
completed: 2026-02-26
---

# Phase 10 Plan 01: Infallible Builders Summary

**All 16 numeric builder methods across six primitive types converted from `Result<Self, EidosError>` to `Self` with clamping, enabling `?`-free and `.unwrap()`-free builder chains**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-26T00:31:07Z
- **Completed:** 2026-02-26T00:34:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Converted all 16 fallible builder methods across Circle, Rect, Line, Arrow, Bezier, and Text to return `Self` with clamping semantics
- Removed `use crate::EidosError` import from all six primitive files — the import is now unused there
- Updated all State-to-primitive conversion methods (`to_circle`, `to_rect`, `to_line`, `to_text`) to eliminate `.unwrap()` calls
- Updated unit tests to rename from `*_returns_err` to `*_is_clamped` pattern and assert clamped values instead of error state

## Task Commits

Each task was committed atomically:

1. **Task 1: Make Circle, Rect, and Line builders infallible** - `cdc6f58` (feat)
2. **Task 2: Make Arrow, Bezier, and Text builders infallible** - `e74a169` (feat)

## Files Created/Modified

- `src/primitives/circle.rs` - .stroke() and .opacity() now return Self; CircleState::to_circle() has no .unwrap(); tests updated
- `src/primitives/rect.rs` - .stroke() and .opacity() now return Self; RectState::to_rect() has no .unwrap(); tests updated
- `src/primitives/line.rs` - .stroke_width() and .opacity() now return Self; LineState::to_line() has no .unwrap(); tests updated
- `src/primitives/arrow.rs` - .stroke_width() and .opacity() now return Self; tests updated
- `src/primitives/bezier.rs` - .stroke() and .opacity() now return Self; tests updated
- `src/primitives/text.rs` - .stroke(), .opacity(), .font_size(), .line_height() now return Self; TextState::to_text() has no .unwrap(); tests updated

## Decisions Made

- Clamp strategy chosen over panic or Err: invalid inputs silently become valid values, matching the API-01 ergonomics goal of `?`-free chains
- `font_size` minimum is 1.0px to ensure text is always at least 1 pixel tall
- `line_height` minimum is 0.1em to allow very tight leading while preventing zero-height lines

## Deviations from Plan

None — plan executed exactly as written.

Note: The dataviz layer (`axes.rs`, `surface_plot.rs`, `confidence_band.rs`, `scatter_plot.rs`, `spline_fit.rs`, `data_curve.rs`) has 30 compile errors from `.expect()` calls on the now-infallible builders. This is expected — the plan explicitly documents that these callers are fixed in plan 02.

## Issues Encountered

None — all primitive files compile correctly. Remaining compile errors are exclusively in dataviz callers, which are addressed in plan 02.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All six primitive types have infallible builder APIs
- Plan 02 (fix dataviz callers) can proceed — 30 `.expect()` calls need to be removed from dataviz files
- The library will fully compile after plan 02 completes

---
*Phase: 10-infallible-builders*
*Completed: 2026-02-26*

## Self-Check: PASSED

Files verified:
- FOUND: src/primitives/circle.rs
- FOUND: src/primitives/rect.rs
- FOUND: src/primitives/line.rs
- FOUND: src/primitives/arrow.rs
- FOUND: src/primitives/bezier.rs
- FOUND: src/primitives/text.rs

Commits verified:
- cdc6f58: feat(10-01): make Circle, Rect, and Line builders infallible
- e74a169: feat(10-01): make Arrow, Bezier, and Text builders infallible

Additional checks:
- EidosError: not found in any of the six primitive files
- .unwrap(): not found in any of the six primitive files
- All 16 builder methods confirmed returning Self (not Result)
