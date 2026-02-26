---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: 3D Surface Visualization
status: unknown
last_updated: "2026-02-26T02:47:27.014Z"
progress:
  total_phases: 3
  completed_phases: 3
  total_plans: 8
  completed_plans: 8
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-25)

**Core value:** A Rust-native way to produce beautiful, animated data visualizations with a declarative API — no Python, no GUI, just code that describes a scene and produces a video.
**Current focus:** Phase 8 in progress — Scatter Points (Plan 01 done: ScatterPlot struct, depth-sorted circles, fade animation; Plan 02 next: SceneBuilder wiring)

## Current Position

Phase: 8 of 8 (Scatter Points) — IN PROGRESS
Plan: 1/2 complete (08-01: ScatterPlot struct + depth-sorted circle rendering)
Status: 08-01 complete — SCAT-01, SCAT-02 core logic implemented; 119 lib tests passing
Last activity: 2026-02-26 — 08-01 complete — ScatterPlot::to_depth_sorted_circles, to_depth_sorted_circles_at, fade animation

Progress: [█████████░] ~90%

## Performance Metrics

**Velocity (v1.0 reference):**
- Total plans completed (v1.0): 19
- Average duration: ~3 min/plan
- Total execution time: ~1 hour

**By Phase (v1.0):**

| Phase | Plans | Avg/Plan |
|-------|-------|----------|
| 01–04.6 (v1.0) | 19 | ~3 min |

**v1.1 velocity:** 3 plans completed.

| Phase | Plans | Avg/Plan |
|-------|-------|----------|
| 05-01 (camera.rs) | 1 | ~2 min |
| 05-02 (surface_plot.rs) | 1 | ~2 min |
| 05-03 (public API wiring) | 1 | ~1 min |

*Updated after each plan completion*
| Phase 06 P01 | 14 | 2 tasks | 4 files |
| Phase 06 P02 | 4 | 2 tasks | 3 files |
| Phase 06 P03 | 3 | 2 tasks | 2 files |
| Phase 07 P01 | 3 | 2 tasks | 1 file |
| Phase 07 P02 | 2 | 2 tasks | 2 files |
| Phase 08 P01 | 2 | 2 tasks | 3 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Key architectural decisions for v1.1 (from research):

- [v1.1 Architecture]: Camera owns the entire data-to-screen transform chain — no projection logic outside `camera.rs`
- [v1.1 Architecture]: CameraState uses spherical coordinates (azimuth_deg, elevation_deg, distance), not Cartesian — avoids gimbal lock, enables clean Tween<CameraState> orbit
- [v1.1 Architecture]: Painter's algorithm (back-to-front centroid sort + backface culling) for face occlusion — correct for approximately-convex GAM surfaces
- [v1.1 Architecture]: SurfacePlot is self-contained; SceneBuilder never holds a Camera — mirrors Axes pattern
- [v1.1 Research flag]: Phase 6 axis edge selection — which of 12 bounding-box edges to show for given camera angle has sparse SVG documentation; short research spike recommended before axis implementation

Decisions from 05-01 execution:

- [05-01]: Z-up spherical coordinates: eye = (distance*cos(el)*sin(az), distance*cos(el)*cos(az), distance*sin(el)); NaVec3::z() as up in look_at_rh
- [05-01]: View matrix precomputed at Camera::new; Perspective3 rebuilt per project_to_screen call (viewport aspect ratio varies)
- [05-01]: is_face_visible uses raw world-space eye position for dot product — only sign matters, normalization unnecessary
- [05-01]: camera.rs re-exports deferred to Plan 03 — pub mod camera declared in dataviz/mod.rs but no crate-root re-export yet

Decisions from 05-02 execution:

- [05-02]: SurfacePlot normalizes at construction; world_point is a pure Vec index lookup — no runtime math
- [05-02]: Temporary pub mod surface_plot in dataviz/mod.rs — Plan 03 will add crate-root re-export
- [05-02]: normalize() span < 1e-12 degenerate guard matches axes.rs convention

Decisions from 05-03 execution:

- [05-03]: Phase 5 types added alphabetically to existing pub use dataviz::{} line in lib.rs — consistent with v1.0 ergonomics
- [05-03]: dataviz/mod.rs pub mod declarations ordered alphabetically (camera before confidence_band, surface_plot at end)
- [05-03]: SURF-01 fully satisfied — use eidos::Camera and use eidos::SurfacePlot are valid import paths
- [Phase 06-01]: Viridis LUT test: b>100 incompatible with canonical data (b=84); changed to b>g && b>50
- [Phase 06-01]: data_extents captured before normalize_to_world_space — only correct placement for axis tick labels
- [Phase 06-01]: eye_position() recomputes from spherical params at call time — consistent with Camera::new formula
- [Phase 06]: Test data for to_primitives must use flat surface (all z=0) for predictable face normals — slanted surfaces have normals sensitive to camera angle
- [Phase 06]: Painter's algorithm: precompute projected corner grid, backface cull via cross-product normal + dot product, sort back-to-front by squared centroid distance

Decisions from 07-02 execution:

- [07-02]: add_surface_at signature appends t_secs as final parameter after viewport — mirrors add_surface shape
- [07-02]: camera_orbit_only_renders_to_mp4 uses add_surface (not add_surface_at) to verify backward compatibility during orbit
- [07-02]: Three distinct test functions for combined/morph-only/orbit-only: separate failure attribution

Decisions from 07-01 execution:

- [07-01]: fitted_zs snapshot at SurfacePlot::new() — immutable; enables to_primitives_at(&self) safe for Fn closures
- [07-01]: z_at() hold semantics: before first window=0.0 (flat), after last=fitted_z, gap between ranges=fitted_z
- [07-01]: camera_at() returns None with no animations — caller uses static camera for no-animation case
- [07-01]: FitAnimation and CameraAnimation are private structs — internal animation range records only

Decisions from 08-01 execution:

- [08-01]: ScatterPlot uses exponential depth falloff (-3t exponent) — visually smoother than linear falloff
- [08-01]: render_circles() private helper shared by both public methods — avoids code duplication
- [08-01]: BEHIND_SURFACE_DIM=0.17, ALPHA_FLOOR=0.03 — locked range constants from CONTEXT.md
- [08-01]: ScatterPlot exported from dataviz/mod.rs alphabetically consistent with existing pattern

Decisions from 06-03 execution:

- [06-03]: far_floor_corner uses integer cast (az as u32) for 4-quadrant match — avoids floating-point edge cases at boundaries; 360.0 normalizes to 0 via modulo
- [06-03]: tick_precision guard: step <= 0.0 returns 0 — prevents log10(0) = -inf panic when surface z range is degenerate (all values equal)
- [06-03]: Stale tests updated: to_primitives now always includes axis primitives (Line + Text) — tests assert Bezier count (face count) instead of total count

### Pending Todos

None yet.

### Blockers/Concerns

- [Phase 6]: 3D axis bounding-box edge selection RESOLVED — quadrant-based floor corner selection implemented, works correctly
- [Phase 6]: 30x30 performance benchmark RESOLVED — to_primitives() runs sub-millisecond (well under 500ms limit); no mitigation needed

## Session Continuity

Last session: 2026-02-26
Stopped at: Completed 08-01-PLAN.md — ScatterPlot struct created; to_depth_sorted_circles, to_depth_sorted_circles_at implemented with exponential depth falloff, behind-surface dimming, and fade animation. 4 scatter tests + 115 prior = 119 total passing. SCAT-01 and SCAT-02 core logic done.
Resume file: None
