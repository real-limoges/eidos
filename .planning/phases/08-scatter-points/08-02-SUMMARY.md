---
phase: 08-scatter-points
plan: 02
subsystem: scene-scatter-integration
tags: [scatter, scene-builder, depth-sort, painter-algorithm, integration-test]
dependency_graph:
  requires:
    - 08-01  # ScatterPlot struct with to_depth_sorted_circles_at
  provides:
    - SceneBuilder::add_scatter / add_scatter_at
    - SceneBuilder::face_depths / prim_depths fields
    - SurfacePlot::visible_face_depths / visible_face_depths_at
    - use eidos::ScatterPlot crate-root export
    - SCAT-01 (static scatter MP4)
    - SCAT-02 (animated scatter MP4)
  affects:
    - src/scene.rs
    - src/dataviz/surface_plot.rs
    - src/lib.rs
tech_stack:
  added: []
  patterns:
    - Painter's algorithm depth-merge (O(n+m) sorted merge of surface faces and scatter circles)
    - prim_depths parallel vec to primitives for depth-tagged merge
    - face_depths accumulated by add_surface for downstream scatter occlusion
key_files:
  created:
    - tests/scatter_points.rs
  modified:
    - src/scene.rs
    - src/dataviz/surface_plot.rs
    - src/lib.rs
decisions:
  - "[08-02]: SceneBuilder carries prim_depths Vec<f64> parallel to primitives — enables O(n+m) merge with scatter circles"
  - "[08-02]: add_surface/add_surface_at populate face_depths via visible_face_depths() — no SVG work, fast lightweight depth-only loop"
  - "[08-02]: Non-surface primitives (axes, labels, add()) get prim_depths = NEG_INFINITY — always painted on top"
  - "[08-02]: add_scatter and add_scatter_at are identical — alias for ergonomic consistency with add_surface_at"
  - "[08-02]: merge_scatter clones circle primitives — Primitive derives Clone; avoids complex ownership gymnastics"
metrics:
  duration: "3 min"
  completed_date: "2026-02-26"
  tasks_completed: 2
  files_modified: 4
---

# Phase 8 Plan 02: ScatterPlot Scene Wiring Summary

**One-liner:** ScatterPlot wired into SceneBuilder with painter's algorithm depth-merge via prim_depths/face_depths, enabling depth-sorted scatter overlays on 3D surfaces in MP4 output.

## What Was Built

### Task 1: SceneBuilder depth-merge infrastructure and scatter methods

**src/dataviz/surface_plot.rs** — Added two new public methods:

- `visible_face_depths(camera, viewport) -> Vec<f64>`: runs the same backface cull loop as `to_primitives()` but only returns centroid depth_sq values. Fast — no SVG/Bezier work.
- `visible_face_depths_at(camera, viewport, t_secs) -> Vec<f64>`: same but with animated z-values (mirrors `to_primitives_at`).

**src/scene.rs** — SceneBuilder struct extended with two new fields:
```rust
pub struct SceneBuilder {
    primitives:  Vec<Primitive>,
    prim_depths: Vec<f64>,   // depth_sq parallel to primitives; NEG_INFINITY = always on top
    face_depths: Vec<f64>,   // visible surface face depths for scatter occlusion
}
```

New/updated SceneBuilder methods:
- `add()`: now pushes `NEG_INFINITY` depth alongside primitive
- `add_axes()`: same — axis/label primitives always on top
- `add_surface()`: calls `visible_face_depths()`, sorts depths back-to-front to tag face primitives; assigns `NEG_INFINITY` to axis overlay primitives
- `add_surface_at()`: same using `visible_face_depths_at()`
- `add_scatter(scatter, camera, viewport, t_secs)`: calls `scatter.to_depth_sorted_circles_at()` with accumulated `face_depths`, then calls `merge_scatter()`
- `add_scatter_at(...)`: alias for `add_scatter`
- `merge_scatter(circles)` (private): O(n+m) merge of two back-to-front sorted lists

**src/lib.rs** — Added `ScatterPlot` to crate-root pub use (alphabetically between `RenderMode` and `SplineFit`).

### Task 2: Integration tests (tests/scatter_points.rs)

Two tests created:

- `static_scatter_renders_to_mp4` (SCAT-01): 5-point scatter overlay on an 8×8 paraboloid, 1s static video at 24fps → MP4 > 1000 bytes
- `animated_scatter_renders_to_mp4` (SCAT-02): 7-point scatter with `animate_fade(3.0, 5.0)` on a morphing paraboloid with `animate_fit(0.0, 3.0)`, 5s video → MP4 > 1000 bytes

## Deviations from Plan

None — plan executed exactly as written. The merge_scatter implementation matched the plan's O(n+m) merge strategy. The face_depths snapshot/sort approach worked cleanly with `to_primitives`'s back-to-front sorted output.

## Verification Results

```
cargo test --lib:   119 passed (all prior tests intact)
cargo test --test scatter_points: 2 passed (SCAT-01, SCAT-02)
cargo test (full suite): all passed (no regressions)
```

## Self-Check: PASSED

Files created/modified:
- FOUND: /Users/reallimoges/repositories/eidos/src/scene.rs
- FOUND: /Users/reallimoges/repositories/eidos/src/dataviz/surface_plot.rs
- FOUND: /Users/reallimoges/repositories/eidos/src/lib.rs
- FOUND: /Users/reallimoges/repositories/eidos/tests/scatter_points.rs

Commits:
- FOUND: 1599b77 (feat: SceneBuilder wiring + crate exports)
- FOUND: fc101f8 (feat: integration tests)
