---
phase: 06-static-3d-surface-rendering
plan: 03
subsystem: dataviz
tags: [surface-plot, axes, tick-generation, painter's-algorithm, 3d-projection, floor-corner-selection]

requires:
  - phase: 06-02
    provides: SurfacePlot::to_primitives(), data_extents(), Camera::project_to_screen()
  - phase: 06-01
    provides: generate_ticks, format_tick, tick_precision in axes.rs; Camera.azimuth_deg field

provides:
  - draw_axes() private helper integrated into SurfacePlot::to_primitives()
  - far_floor_corner(azimuth_deg) quadrant-based floor corner selection
  - generate_ticks, format_tick, tick_precision made pub(crate) in axes.rs
  - 3D X/Y/Z axis lines, tick marks, tick value labels, axis name labels in all to_primitives() output
affects: []

tech-stack:
  added: []
  patterns: [azimuth-quadrant floor corner selection, perpendicular screen-space tick direction, degenerate tick step guard]

key-files:
  created: []
  modified:
    - src/dataviz/surface_plot.rs
    - src/dataviz/axes.rs

key-decisions:
  - "far_floor_corner uses integer cast (az as u32) for 4-quadrant match — avoids floating-point edge cases at quadrant boundaries; 360.0 normalizes to 0 via modulo"
  - "tick_precision guard: step <= 0.0 returns 0 — prevents log10(0) = -inf panic when data range is degenerate (all values equal, step=0)"
  - "Stale tests (to_primitives_2x2_produces_one_face, to_primitives_wireframe_mode) updated to assert Bezier count instead of total count — now that axes are always appended, total primitive count includes Line + Text axis elements"
  - "Tick world position mapping: t = (tick_val - data_min) / (data_max - data_min) * 2.0 - 1.0 maps data-space tick to normalized [-1,1] world; then linearly interpolate along axis segment"

patterns-established:
  - "Perpendicular tick direction: compute screen-space perpendicular to axis direction (perp_dx = -(s1.y-s0.y), perp_dy = s1.x-s0.x), normalize, scale by TICK_HALF_LEN"
  - "Axis label placement: end of axis + signum() * offset to always place label in the outward direction regardless of axis orientation"
  - "Degenerate guard pattern: step <= 0 returns safe fallback (0 precision) to handle flat surfaces where all z are equal"

requirements-completed: [SURF-04]

duration: ~3min
completed: 2026-02-26
---

# Phase 6 Plan 03: 3D Axis Rendering Summary

**Camera-azimuth-aware 3D axis rendering with Heckbert tick marks and data-space labels integrated into SurfacePlot::to_primitives() via draw_axes() helper**

## Performance

- **Duration:** ~3 minutes
- **Started:** 2026-02-26T01:52:44Z
- **Completed:** 2026-02-26T01:56:08Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `generate_ticks`, `tick_precision`, `format_tick` made `pub(crate)` in `axes.rs` — reusable by surface plot axis rendering
- `far_floor_corner(azimuth_deg)` selects the bounding-box floor corner most visible for the current camera azimuth (4 discrete quadrants)
- `draw_axes()` renders 3 axis lines (X/Y/Z) from the selected floor corner, with perpendicular tick marks, data-space tick value labels, and axis name labels
- Axes appended at the end of `to_primitives()` output — render on top of surface faces
- Fixed degenerate `tick_precision` when step = 0 (flat surface z axis) to prevent log10(0) = -inf panic
- Total test count: 101 lib tests + integration tests, 0 failures; added 7 new tests

## Task Commits

Each task was committed atomically:

1. **Task 1: Make generate_ticks/format_tick/tick_precision pub(crate)** - `d60e44b` (feat)
2. **Task 2: Implement 3D axis rendering in to_primitives()** - `1ff2edb` (feat)

## Files Created/Modified

- `src/dataviz/surface_plot.rs` - Added `draw_axes()` method, `far_floor_corner()` fn, imports for Line/Text/generate_ticks/format_tick; updated `to_primitives()` to append axis primitives; added 7 new tests; updated 2 stale test assertions
- `src/dataviz/axes.rs` - Changed `generate_ticks`, `tick_precision`, `format_tick` from `fn` to `pub(crate) fn`; fixed `tick_precision` degenerate step guard

## Decisions Made

1. **Integer cast for quadrant matching**: `az as u32` in `far_floor_corner` avoids floating-point boundary cases and gives clean match arm ranges. 360.0 normalizes to 0 via `% 360.0`, matching Q1.

2. **Degenerate tick step guard in tick_precision**: When a surface has all z equal (flat), `data_extents()` returns `z_data_min == z_data_max`. `generate_ticks(0.0, 0.0, 5)` early-returns `vec![0.0, 0.0]`, giving `z_step = 0.0`. Without the guard, `(-0.0_f64.log10().floor()) as usize` = `(-(-inf)) as usize` panics. Added `if step <= 0.0 { return 0; }`.

3. **Updated stale test assertions**: Two tests (`to_primitives_2x2_produces_one_face` and `to_primitives_wireframe_mode`) were written before axes were integrated. They expected exactly 1 primitive or all-Bezier primitives respectively. Updated to assert Bezier count (face count) instead of total count, since axes now add Line and Text elements to every `to_primitives()` call.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] tick_precision(0.0) panics with "Formatting argument out of range"**
- **Found during:** Task 2 (running tests after to_primitives integration)
- **Issue:** `make_2x2_plot()` uses all-zero z values → `z_data_min == z_data_max == 0.0` → `generate_ticks(0.0, 0.0, 5)` returns `[0.0, 0.0]` → `z_step = 0.0` → `tick_precision(0.0)` computes `(-0.0f64.log10().floor()) as usize` = `usize::MAX` → `format!("{:.precision$}", ...)` panics
- **Fix:** Added `if step <= 0.0 { return 0; }` guard and `.clamp(0.0, 15.0)` on the log10 result in `tick_precision`
- **Files modified:** `src/dataviz/axes.rs`
- **Verification:** All 5 previously failing to_primitives tests now pass
- **Committed in:** `1ff2edb` (Task 2 commit)

**2. [Rule 1 - Bug] Stale test assertions for to_primitives() after axis integration**
- **Found during:** Task 2 (running tests after draw_axes integration)
- **Issue:** `to_primitives_2x2_produces_one_face` expected `len() == 1` (face only); now gets 35 (face + 34 axis primitives). `to_primitives_wireframe_mode` expected all primitives to be Bezier; now also contains Line and Text axis elements.
- **Fix:** Updated both tests to assert Bezier count (1 face) rather than total count; added assertion that total count > 1 to verify axes are present
- **Files modified:** `src/dataviz/surface_plot.rs`
- **Verification:** Both tests now pass
- **Committed in:** `1ff2edb` (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (both Rule 1 - Bug)
**Impact on plan:** Both fixes were necessary for correctness. The tick_precision bug was latent in axes.rs and only manifested when called with a degenerate step from surface plot data. No scope creep.

## Issues Encountered

None beyond the two auto-fixed deviations above.

## Next Phase Readiness

- Phase 6 complete: SURF-02 (wireframe), SURF-03 (shaded), SURF-04 (axes) all implemented
- `SurfacePlot::to_primitives()` produces a full static 3D scene: shaded or wireframe faces + labeled axes
- `eidos::SurfacePlot`, `eidos::Camera`, `eidos::RenderMode` all available at crate root
- `SceneBuilder::add_surface()` wires the pipeline into scene builder for frame rendering
- No architectural blockers identified for future phases

---
*Phase: 06-static-3d-surface-rendering*
*Completed: 2026-02-26*
