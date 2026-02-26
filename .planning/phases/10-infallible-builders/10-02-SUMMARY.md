---
phase: 10-infallible-builders
plan: "02"
subsystem: api

tags: [rust, dataviz, builder-pattern, infallible, clamping, examples, tests]

requires:
  - phase: 10-01
    provides: "All six primitive builder methods now return Self with clamping — no more Result<Self, EidosError>"

provides:
  - "All dataviz internals free of .expect() on primitive builder calls (axes.rs, surface_plot.rs, scatter_plot.rs)"
  - "DataCurve::stroke() and DataCurve::opacity() converted to infallible builders with clamping"
  - "ConfidenceBand::opacity() converted to infallible builder with clamping"
  - "SplineFit::stroke_width() converted to infallible builder with clamping"
  - "All examples compile without .unwrap() or ? on builder methods"
  - "All integration tests updated to verify clamping instead of error returns"
  - "cargo test passes with zero failures"

affects:
  - downstream callers of DataCurve, ConfidenceBand, SplineFit builder APIs

tech-stack:
  added: []
  patterns:
    - "Infallible builder pattern propagated to dataviz layer: DataCurve, ConfidenceBand, SplineFit"
    - "All builder chains in examples now termination-free: no ? or .unwrap()"

key-files:
  created: []
  modified:
    - src/dataviz/axes.rs
    - src/dataviz/surface_plot.rs
    - src/dataviz/data_curve.rs
    - src/dataviz/confidence_band.rs
    - src/dataviz/scatter_plot.rs
    - src/dataviz/spline_fit.rs
    - examples/basic_scene.rs
    - examples/data_plot.rs
    - examples/gam_plot.rs
    - tests/data_viz.rs
    - tests/gam_viz.rs

key-decisions:
  - "DataCurve::stroke(), DataCurve::opacity(), ConfidenceBand::opacity(), SplineFit::stroke_width() all converted to infallible with clamping — consistent with primitive layer pattern"
  - "EidosError import retained in data_curve.rs, confidence_band.rs — still needed by ::new() validation (point count)"

patterns-established:
  - "Dataviz builder methods mirror primitive pattern: pub fn stroke(mut self, color: Color, width: f64) -> Self { ... width.max(0.0) ... }"
  - "Test naming: *_is_clamped asserts value, not *_returns_err on .is_err()"

requirements-completed:
  - API-01

duration: 6min
completed: 2026-02-26
---

# Phase 10 Plan 02: Infallible Builders Summary

**All dataviz callers updated to use infallible builder chains — 30 .expect() calls removed, 4 dataviz builder methods converted from Result<Self> to Self with clamping, cargo test passes with zero failures**

## Performance

- **Duration:** 6 min
- **Started:** 2026-02-26T00:36:18Z
- **Completed:** 2026-02-26T00:42:30Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments

- Removed all 30 `.expect()` calls from dataviz internals that were chained after primitive builder methods (axes.rs, surface_plot.rs, scatter_plot.rs)
- Converted `DataCurve::stroke()`, `DataCurve::opacity()`, `ConfidenceBand::opacity()`, and `SplineFit::stroke_width()` from `Result<Self, EidosError>` to `Self` with clamping — matching the primitive layer's infallible pattern
- Updated all examples (`basic_scene.rs`, `data_plot.rs`, `gam_plot.rs`) to remove `.unwrap()` and `?` from builder chains
- Updated integration tests (`tests/data_viz.rs`, `tests/gam_viz.rs`) to verify clamping instead of error returns

## Task Commits

Each task was committed atomically:

1. **Task 1: Update dataviz internals — remove .expect() from primitive builder calls** - `a3b8bf8` (feat)
2. **Task 2: Update examples and integration tests — remove .unwrap() and ? from builder chains** - `027e22b` (feat)

## Files Created/Modified

- `src/dataviz/axes.rs` - All .expect() chains after stroke_width(), font_size(), stroke(), opacity() removed
- `src/dataviz/surface_plot.rs` - All .expect() chains after stroke() in wireframe/shaded rendering and draw_axes() X/Y/Z sections removed
- `src/dataviz/data_curve.rs` - stroke() and opacity() converted to infallible Self-returning builders; tests renamed *_is_clamped
- `src/dataviz/confidence_band.rs` - opacity() converted to infallible Self-returning builder; test renamed *_is_clamped
- `src/dataviz/scatter_plot.rs` - .expect() removed from Circle::opacity() call
- `src/dataviz/spline_fit.rs` - stroke_width() converted to infallible Self-returning builder; .expect() removed from to_bezier()
- `examples/basic_scene.rs` - .unwrap() removed from all builder calls
- `examples/data_plot.rs` - ? removed from DataCurve::stroke() calls
- `examples/gam_plot.rs` - ? removed from ConfidenceBand::opacity(), DataCurve::stroke(), SplineFit::stroke_width()
- `tests/data_viz.rs` - data_curve_negative_stroke_width_err updated to assert clamping
- `tests/gam_viz.rs` - .expect() removed from ConfidenceBand::opacity() and SplineFit::stroke_width()

## Decisions Made

- `EidosError` import retained in `data_curve.rs` and `confidence_band.rs` — still needed for `::new()` point-count validation (returning Result for insufficient data is correct behavior; infallible conversion only applies to numeric builder methods)
- `SplineFit::stroke_width()` also converted to infallible despite not being in the plan's file list — it was blocking compilation and followed the same pattern (Rule 3 auto-fix)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed spline_fit.rs — had same .expect() and fallible stroke_width() pattern**
- **Found during:** Task 1 (building --lib revealed 6 errors in spline_fit.rs)
- **Issue:** `spline_fit.rs` had `.expect("stroke validated at construction")` in `to_bezier()` and `stroke_width()` returning `Result<Self, EidosError>` — same pattern as files listed in the plan but omitted from the file list
- **Fix:** Converted `stroke_width()` to infallible with clamping; removed `.expect()` from `to_bezier()`
- **Files modified:** `src/dataviz/spline_fit.rs`
- **Verification:** `cargo build --lib` succeeded after fix
- **Committed in:** `a3b8bf8` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The fix was essential — library would not compile without it. spline_fit.rs was simply missing from the plan's file list despite having the identical pattern.

## Issues Encountered

None — once spline_fit.rs was added to scope, the changes were mechanical.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The infallible builder refactor (API-01) is fully complete across the entire codebase
- `cargo test` passes with zero failures
- `cargo build --examples` succeeds
- No `.unwrap()`, `.expect()`, or `?` exists anywhere in src/, examples/, or tests/ after any builder method
- The builder chain `Circle::new(cx, cy, r).fill(Color::RED).opacity(0.5).stroke(Color::WHITE, 2.0)` returns `Circle` with no error handling

---
*Phase: 10-infallible-builders*
*Completed: 2026-02-26*
