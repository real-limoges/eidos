---
phase: 08-scatter-points
plan: "01"
subsystem: dataviz
tags: [scatter, rendering, depth-sorting, animation, opacity]
depends_on: []
provides: [ScatterPlot, depth_sorted_circles]
affects: [src/dataviz/scatter_plot.rs, src/dataviz/surface_plot.rs]
tech_stack:
  added: []
  patterns: [exponential-depth-falloff, painter-algorithm-merge-ready, builder-pattern]
key_files:
  created:
    - src/dataviz/scatter_plot.rs
  modified:
    - src/dataviz/surface_plot.rs
    - src/dataviz/mod.rs
decisions:
  - "ScatterPlot uses exponential depth falloff (-3t exponent) — visually smoother than linear falloff"
  - "BEHIND_SURFACE_DIM = 0.17 — within locked 15-20% range from CONTEXT.md"
  - "ALPHA_FLOOR = 0.03 — hard floor prevents completely invisible primitives"
  - "render_circles() private helper shared by both to_depth_sorted_circles and to_depth_sorted_circles_at — avoids code duplication"
  - "ScatterPlot exported from dataviz/mod.rs — consistent with existing pub use pattern"
metrics:
  duration: "~2 min"
  completed: "2026-02-26"
  tasks_completed: 2
  files_created: 1
  files_modified: 2
requirements-completed: [SCAT-01, SCAT-02]
---

# Phase 08 Plan 01: ScatterPlot Struct and Depth-Sorted Circle Rendering Summary

**One-liner:** ScatterPlot struct with exponential depth-falloff opacity, behind-surface dimming, and linear fade animation — produces depth-tagged Circle primitives ready for painter's algorithm merge.

## What Was Built

Implemented the core scatter rendering layer for phase 08. Two tasks executed:

**Task 1 — normalize() pub(crate):** Changed the visibility of the `normalize()` helper in `surface_plot.rs` from private to `pub(crate)`, enabling `scatter_plot.rs` to import it for coordinate normalization. No other changes; all 115 existing tests continued to pass.

**Task 2 — ScatterPlot implementation:** Created `src/dataviz/scatter_plot.rs` with:

- `ScatterPlot::new(points, surface_extents)` — accepts raw `(x,y,z)` data and a 6-tuple extents from `SurfacePlot::data_extents()` for normalization
- `world_point()` — private helper normalizing raw data coords to world space `[-1, 1]` via `surface_plot::normalize()`
- `depth_opacity()` — exponential falloff (`(-3t).exp()`) mapping depth range to `[1.0, ~0.25]` with a `FLOOR = 0.25`
- `fade_at(t)` — linear fade animation: 0.0 before start, 1.0 after end, linear in between
- `to_depth_sorted_circles()` — static rendering returning `Vec<(f64, Primitive)>`
- `to_depth_sorted_circles_at()` — animated rendering; skips all points when `fade == 0.0`
- Builder methods: `with_color()`, `with_radius()`, `animate_fade()`
- Default color `Color::rgb(255, 120, 50)` (warm orange, visible against viridis colormap), default radius `4.5px`

## Test Results

- 4 new scatter unit tests: all pass
  - `scatter_fade_at_interpolates` — before/midpoint/after assertions
  - `scatter_static_fade_is_one` — no animation attached always returns 1.0
  - `scatter_circles_at_empty_before_fade_start` — zero circles before fade window
  - `scatter_single_point_produces_one_circle` — single visible point → one Circle primitive
- 115 pre-existing lib tests: all continue to pass
- Total: 119 lib tests passing, 0 failed

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 110bb27 | feat(08-01): make normalize() pub(crate) in surface_plot.rs |
| 2 | 4892651 | feat(08-01): implement ScatterPlot in src/dataviz/scatter_plot.rs |

## Deviations from Plan

**Auto-additions beyond plan spec:**

1. **[Rule 2 - Missing functionality] Added `render_circles()` private helper**
   - Found during: Task 2 implementation
   - Issue: `to_depth_sorted_circles()` and `to_depth_sorted_circles_at()` share identical projection/opacity logic
   - Fix: Extracted shared logic into `render_circles(camera, viewport, face_depths, fade)` called by both public methods
   - Files modified: src/dataviz/scatter_plot.rs
   - Commit: 4892651

2. **[Rule 2 - Missing re-export] Added ScatterPlot to dataviz/mod.rs**
   - Found during: Task 2 implementation
   - Issue: Plan specified creating the file but the module would not be accessible without registration
   - Fix: Added `pub mod scatter_plot` declaration and `pub use scatter_plot::ScatterPlot` re-export
   - Files modified: src/dataviz/mod.rs
   - Commit: 4892651

## Self-Check: PASSED

- src/dataviz/scatter_plot.rs exists: FOUND
- src/dataviz/surface_plot.rs contains `pub(crate) fn normalize`: FOUND
- Commit 110bb27 exists: FOUND
- Commit 4892651 exists: FOUND
- 119 lib tests pass: VERIFIED
