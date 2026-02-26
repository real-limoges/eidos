---
phase: 06-static-3d-surface-rendering
plan: 01
subsystem: dataviz
tags: [colormap, viridis, camera, surface-plot, render-mode, data-extents]
dependency_graph:
  requires: [05-03]
  provides: [colormap.viridis_color, camera.eye_position, surface_plot.RenderMode, surface_plot.data_extents, surface_plot.builder_methods]
  affects: [06-02, 06-03]
tech_stack:
  added: []
  patterns: [256-entry LUT colormap, builder pattern for rendering config, spherical-to-cartesian eye_position]
key_files:
  created:
    - src/dataviz/colormap.rs
  modified:
    - src/dataviz/camera.rs
    - src/dataviz/surface_plot.rs
    - src/dataviz/mod.rs
decisions:
  - "Viridis LUT index 0 = (0.267004, 0.004874, 0.329415) converts to b=84 u8, not b>100 — adjusted test to b > g and b > 50"
  - "RenderMode::Shaded is the default via Default trait impl"
  - "eye_position() recomputes from spherical params at call time — same formula as Camera::new"
  - "data_extents() stores raw pre-normalization values — Plan 03 axis ticks will use these for label generation"
metrics:
  duration: "~14 minutes"
  completed: "2026-02-26"
  tasks_completed: 2
  files_changed: 4
  tests_added: 12
  tests_total: 111
---

# Phase 6 Plan 01: Surface Rendering Foundations Summary

Laid the foundational pieces that surface rendering (Plan 02) and axis rendering (Plan 03) both depend on: the viridis colormap module, `Camera::eye_position()`, data-extent storage in SurfacePlot, the `RenderMode` enum, and SurfacePlot builder methods for rendering configuration.

## Tasks Completed

| Task | Description | Commit | Status |
|------|-------------|--------|--------|
| 1 | Create colormap.rs with 256-entry viridis LUT | 34682f8 | Done |
| 2 | Add Camera::eye_position(), RenderMode, SurfacePlot data extents + builders | a68d7ac | Done |

## What Was Built

**colormap.rs (new)**
- `VIRIDIS_LUT: [(f32, f32, f32); 256]` — canonical BIDS/matplotlib viridis palette
- `pub fn viridis_color(t: f64) -> crate::Color` — clamped LUT lookup, t=0 → deep purple, t=1 → bright yellow
- 5 tests: at_zero_is_purple, at_one_is_yellow, clamps_below_zero, clamps_above_one, midpoint_is_teal_green

**camera.rs (modified)**
- `pub fn eye_position(&self) -> (f64, f64, f64)` — returns world-space camera eye position using spherical formula
- 2 tests: eye_position at azimuth=0/elevation=0 on +Y axis, eye_position at elevation=45 has positive z

**surface_plot.rs (modified)**
- `pub enum RenderMode { Shaded, Wireframe, ShadedWireframe }` with `Default::Shaded`
- New private fields: `x/y/z_data_min/max` (pre-normalization data extents), `render_mode`, `x/y/z_label`, `show_base_grid`
- Builder methods: `render_mode()`, `x_label()`, `y_label()`, `z_label()`, `show_base_grid()`
- Accessor methods: `data_extents()`, `render_mode_value()`, `x/y/z_label_value()`, `show_base_grid_value()`
- `SurfacePlot::new()` now captures data extents before calling `normalize_to_world_space()`
- 5 tests: render_mode_default_is_shaded, builder_sets_render_mode, builder_sets_x_label, show_base_grid_default_false, data_extents_captures_original_values

**dataviz/mod.rs (modified)**
- Added `pub mod colormap;` (alphabetically between axes and camera)
- Updated re-export: `pub use surface_plot::{RenderMode, SurfacePlot};`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed viridis_at_zero_is_purple test threshold**
- **Found during:** Task 1
- **Issue:** Plan specified `assert b > 100` but canonical viridis index 0 has b = round(0.329415 * 255) = 84, which is < 100. The test would have failed with the correct viridis data.
- **Fix:** Changed assertion to `b > c.g` (blue dominates over green, characterizing purple hue) and `b > 50` (meaningful blue component). Both conditions hold for the canonical purple endpoint: b=84 > g=1.
- **Files modified:** src/dataviz/colormap.rs
- **Commit:** 34682f8

## Key Decisions

1. **Viridis LUT index 0 purple assertion**: `b > 100` is incompatible with the canonical BIDS viridis data (b=84). Changed to `b > g && b > 50` which correctly characterizes purple without requiring an exact threshold that contradicts the source data.

2. **Data extents captured before normalization**: `SurfacePlot::new()` now calls `min_max()` on all three axes before `normalize_to_world_space()`. This is the only correct placement — post-normalization all values would be in [-1, 1] losing the original scale information needed for axis tick labels.

3. **eye_position() recomputes from spherical params**: Same formula as `Camera::new()`. This ensures no drift between the stored eye vector and the returned position, even though the stored `eye` NaVec3 field could theoretically be returned directly. Using the public API formula maintains consistency.

## Self-Check: PASSED

- colormap.rs: FOUND at src/dataviz/colormap.rs
- camera.rs: FOUND at src/dataviz/camera.rs (with eye_position method)
- surface_plot.rs: FOUND at src/dataviz/surface_plot.rs (with RenderMode, data extents, builders)
- Commit 34682f8: FOUND (feat(06-01): create colormap.rs)
- Commit a68d7ac: FOUND (feat(06-01): add Camera::eye_position(), RenderMode...)
- All 111 tests pass (88 + 12 + 2 + 7 + 2), 0 failures
