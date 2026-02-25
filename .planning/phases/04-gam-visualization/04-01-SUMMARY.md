---
phase: 04-gam-visualization
plan: 01
subsystem: dataviz
tags: [rust, catmull-rom, bezier, spline, confidence-band, gam, dataviz]

# Dependency graph
requires:
  - phase: 03-data-visualization
    provides: DataCurve, Axes, Bezier primitives, Catmull-Rom spline logic
provides:
  - Shared pub(crate) catmull_rom_segment_to_bezier helper in src/dataviz/spline.rs
  - ConfidenceBand struct with fill_color()/opacity() builder API and to_bezier_path()
  - Axes::add_band() builder method and bands field with auto-range integration
  - ConfidenceBand re-exported as public API from lib.rs
affects: [04-gam-visualization/04-02, spline-fit-future-plans]

# Tech tracking
tech-stack:
  added: []
  patterns: [shared-spline-helper, closed-bezier-fill-path, fill-only-band-rendering]

key-files:
  created:
    - src/dataviz/spline.rs
    - src/dataviz/confidence_band.rs
  modified:
    - src/dataviz/data_curve.rs
    - src/dataviz/axes.rs
    - src/dataviz/mod.rs
    - src/lib.rs

key-decisions:
  - "spline.rs is pub(crate) — ConfidenceBand and SplineFit can share catmull_rom_segment_to_bezier without exposing it publicly"
  - "to_bezier_path() takes pre-mapped pixel-space points (same pattern as DataCurve) — caller maps data to visual space"
  - "Band is fill-only (no stroke on bound lines) — Bezier::fill() without .stroke() call; SVG sets stroke=none implicitly via fill-only attribute"
  - "Default opacity 0.25 makes band semi-transparent so data curves stay visually dominant"
  - "Band points feed into to_primitives() Step 1 auto-range so axes scale to fit both curve and band data"
  - "ConfidenceBand added to lib.rs public re-exports alongside Axes/DataCurve — first-class public API"

patterns-established:
  - "Closed bezier band: MoveTo upper[0] + forward Catmull-Rom upper + LineTo lower_rev[0] + reversed Catmull-Rom lower + Close"
  - "Phantom endpoint duplication at array boundaries (p0=p[0] for i==0, p3=p[n-1] for i+2>=n) prevents boundary kinks"

requirements-completed: [GAM-01]

# Metrics
duration: 3min
completed: 2026-02-25
---

# Phase 4 Plan 1: Catmull-Rom Shared Helper and ConfidenceBand Summary

**Catmull-Rom spline extracted to shared spline.rs module; ConfidenceBand renders a closed filled region between upper/lower bound arrays, composable with Axes via add_band()**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-25T16:18:58Z
- **Completed:** 2026-02-25T16:21:42Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Extracted `catmull_rom_segment_to_bezier` from data_curve.rs to `src/dataviz/spline.rs` as `pub(crate)` — now shared by DataCurve, ConfidenceBand, and future SplineFit
- Created `ConfidenceBand` struct with builder API (`fill_color()`, `opacity()`) and `to_bezier_path()` that produces a closed Catmull-Rom filled path
- Integrated `ConfidenceBand` into `Axes` via `add_band()` builder; bands contribute to auto-range and are emitted before data curves (Step 6.5) so curves render on top

## Task Commits

Each task was committed atomically:

1. **Task 1: Extract Catmull-Rom helper to shared spline.rs module** - `0f32105` (refactor)
2. **Task 2: Implement ConfidenceBand and integrate into Axes** - `b1be90b` (feat)

**Plan metadata:** (included in final docs commit)

## Files Created/Modified

- `src/dataviz/spline.rs` - Shared `pub(crate) fn catmull_rom_segment_to_bezier` with 2 unit tests
- `src/dataviz/confidence_band.rs` - ConfidenceBand struct with builder API, to_bezier_path(), 4 unit tests
- `src/dataviz/data_curve.rs` - Removed private catmull_rom fn, added import from spline module
- `src/dataviz/axes.rs` - Added bands field, add_band() builder, Step 6.5 band rendering, Step 1 band auto-range
- `src/dataviz/mod.rs` - Registered pub(crate) mod spline and pub mod confidence_band with re-export
- `src/lib.rs` - Added ConfidenceBand to public re-exports

## Decisions Made

- `Bezier::fill()` already existed in bezier.rs (line 104) — used directly, no modification needed
- Band path uses fill-only (no `.stroke()` call), which leaves `stroke: None` in Bezier; SVG to_svg_element() already handles `None` stroke by omitting the attribute
- ConfidenceBand re-exported from lib.rs to be a first-class public API alongside Axes/DataCurve

## Deviations from Plan

None — plan executed exactly as written. `Bezier::fill()` existence check passed (no addition needed).

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- ConfidenceBand is ready for use; compose with DataCurve and Axes to render GAM fit with confidence interval
- `src/dataviz/spline.rs` is ready to be imported by future `SplineFit` implementation
- All 55 lib tests pass; clean build with no warnings

---
*Phase: 04-gam-visualization*
*Completed: 2026-02-25*
