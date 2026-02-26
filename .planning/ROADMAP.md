# Roadmap: eidos

## Milestones

- ✅ **v1.0** — Phases 1–4.6 (shipped 2026-02-25)
- ✅ **v1.1 3D Surface Visualization** — Phases 5–8 (completed 2026-02-26)

## Phases

<details>
<summary>✅ v1.0 — Phases 1–4.6 — SHIPPED 2026-02-25</summary>

- [x] Phase 1: Rendering Pipeline and Primitives (5/5 plans) — completed 2026-02-25
- [x] Phase 2: Animation Engine (3/3 plans) — completed 2026-02-25
- [x] Phase 2.5: Tech Debt Cleanup (2/2 plans) — completed 2026-02-25
- [x] Phase 3: Data Visualization (3/3 plans) — completed 2026-02-25
- [x] Phase 3.5: Dataviz Tech Debt Cleanup (1/1 plan) — completed 2026-02-25
- [x] Phase 4: GAM Visualization (3/3 plans) — completed 2026-02-25
- [x] Phase 4.5: GAM Visualization Completion (1/1 plan) — completed 2026-02-25
- [x] Phase 4.6: v1.0 API Ergonomics Cleanup (1/1 plan) — completed 2026-02-25

Full phase details: `.planning/milestones/v1.0-ROADMAP.md`

</details>

### 🚧 v1.1 3D Surface Visualization (In Progress)

**Milestone Goal:** Add 3D perspective mesh rendering with fitting animation, camera rotation, and data point scatter — enabling animated GAM (and ML) surface visualizations.

- [x] **Phase 5: Camera and Projection Foundation** - Perspective projection math, Camera/CameraState structs, and the data-to-screen transform chain (completed 2026-02-25)
- [x] **Phase 6: Static 3D Surface Rendering** - Wireframe mesh, shaded surface with z-height colormap, and 3D cartesian axes (completed 2026-02-26)
- [x] **Phase 7: Surface and Camera Animation** - Surface morphing from flat to fitted shape, and camera orbit animation (completed 2026-02-26)
- [x] **Phase 8: Scatter Points** - 3D scatter point rendering with depth-based opacity and fade-in animation (completed 2026-02-26)

## Phase Details

### Phase 5: Camera and Projection Foundation
**Goal**: Users can construct a 3D surface plot with a configurable camera viewpoint and obtain valid 2D screen projections of world-space points
**Depends on**: Phase 4.6 (v1.0 complete)
**Requirements**: SURF-01
**Success Criteria** (what must be TRUE):
  1. User can create a `SurfacePlot` from a regular grid of (x, y, z) data
  2. User can configure the camera viewpoint via azimuth, elevation, and distance parameters
  3. `Camera::project_to_screen()` maps known world-space coordinates to correct pixel coordinates (unit-tested)
  4. Backface culling correctly identifies and discards faces not visible from the configured viewpoint
**Plans**: 3 plans

Plans:
- [x] 05-01-PLAN.md — Camera struct with spherical coordinates and project_to_screen
- [x] 05-02-PLAN.md — SurfacePlot data container with normalized world vertices
- [x] 05-03-PLAN.md — Public API wiring (crate-root re-exports for Camera and SurfacePlot)

### Phase 6: Static 3D Surface Rendering
**Goal**: Users can render a complete, visually correct 3D surface scene — wireframe or shaded — with labeled cartesian axes, to a single video frame
**Depends on**: Phase 5
**Requirements**: SURF-02, SURF-03, SURF-04
**Success Criteria** (what must be TRUE):
  1. User can render a surface as a wireframe mesh with depth-sorted projected edges from any configured viewpoint
  2. User can render a surface as a shaded mesh where face color varies with z-height via a configurable gradient
  3. User can add labeled 3D cartesian axes with projected tick marks to the surface plot
  4. A 30x30 mesh renders to an MP4 frame within an acceptable time budget (per-frame benchmark passes)
**Plans**: 3 plans

Plans:
- [ ] 06-01-PLAN.md — Colormap module, Camera::eye_position(), RenderMode enum, SurfacePlot data extents and builder methods
- [ ] 06-02-PLAN.md — SurfacePlot::to_primitives() painter's algorithm (SURF-02 wireframe + SURF-03 shaded) and SceneBuilder::add_surface()
- [ ] 06-03-PLAN.md — 3D axis rendering integrated into to_primitives() (SURF-04), axis edge selection, tick marks and labels

### Phase 7: Surface and Camera Animation
**Goal**: Users can produce an animated video where the surface morphs from flat to its fitted shape, and the camera orbits around the surface
**Depends on**: Phase 6
**Requirements**: ANIM-01, ANIM-02
**Success Criteria** (what must be TRUE):
  1. User can call `animate_fit(start_time, duration, easing)` and the surface smoothly morphs from a flat plane to the final z-values over that time range
  2. User can animate the camera azimuth sweeping from a start angle to an end angle over a specified time range using any existing Easing variant
  3. An integration test renders a morphing surface with an orbiting camera to a valid MP4 without visual artifacts
**Plans**: 2 plans

Plans:
- [x] 07-01-PLAN.md — SurfacePlot animation infrastructure: fitted_zs, FitAnimation, animate_fit(), z_at(), to_primitives_at(), CameraAnimation, animate_camera_azimuth(), camera_at()
- [x] 07-02-PLAN.md — SceneBuilder::add_surface_at() wiring and integration test (morphing surface + orbiting camera → MP4)

### Phase 8: Scatter Points
**Goal**: Users can overlay raw (x, y, z) data points on a 3D surface plot, with depth-based opacity and optional fade-in animation
**Depends on**: Phase 6
**Requirements**: SCAT-01, SCAT-02
**Success Criteria** (what must be TRUE):
  1. User can add scatter points from a `Vec<(f64, f64, f64)>` and they render as projected circles depth-sorted alongside mesh faces
  2. Scatter points closer to the viewer appear more opaque; points behind the surface are occluded correctly
  3. User can animate scatter points fading in over a specified time range, appearing after the surface is visible
**Plans**: 2 plans

Plans:
- [x] 08-01-PLAN.md — ScatterPlot struct: coordinate normalization, depth opacity, behind-surface dimming, fade animation (SCAT-01, SCAT-02)
- [x] 08-02-PLAN.md — SceneBuilder::add_scatter/add_scatter_at wiring, depth-merge with painter's algorithm, integration tests → MP4

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Rendering Pipeline and Primitives | v1.0 | 5/5 | Complete | 2026-02-25 |
| 2. Animation Engine | v1.0 | 3/3 | Complete | 2026-02-25 |
| 2.5. Tech Debt Cleanup | v1.0 | 2/2 | Complete | 2026-02-25 |
| 3. Data Visualization | v1.0 | 3/3 | Complete | 2026-02-25 |
| 3.5. Dataviz Tech Debt Cleanup | v1.0 | 1/1 | Complete | 2026-02-25 |
| 4. GAM Visualization | v1.0 | 3/3 | Complete | 2026-02-25 |
| 4.5. GAM Visualization Completion | v1.0 | 1/1 | Complete | 2026-02-25 |
| 4.6. v1.0 API Ergonomics Cleanup | v1.0 | 1/1 | Complete | 2026-02-25 |
| 5. Camera and Projection Foundation | v1.1 | 3/3 | Complete | 2026-02-25 |
| 6. Static 3D Surface Rendering | 3/3 | Complete   | 2026-02-26 | - |
| 7. Surface and Camera Animation | 2/2 | Complete   | 2026-02-26 | 2026-02-26 |
| 8. Scatter Points | v1.1 | 2/2 | Complete | 2026-02-26 |
