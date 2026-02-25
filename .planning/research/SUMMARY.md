# Project Research Summary

**Project:** eidos — Manim-inspired Rust animation/visualization library (v1.1: 3D Surface milestone)
**Domain:** 3D surface visualization extension for an existing SVG-based Rust animation library
**Researched:** 2026-02-25
**Confidence:** HIGH

## Executive Summary

eidos v1.1 adds 3D surface visualization to a library that already ships a complete 2D animation pipeline (SVG generation via `svg` crate, rasterization via `resvg`/`tiny-skia`, video encoding via `ffmpeg-sidecar`). The research confirms that no new rendering infrastructure is needed: 3D rendering is entirely handled by projecting world-space coordinates to screen-space 2D primitives inside a new `SurfacePlot::to_primitives()` method, leaving `svg_gen.rs`, the rasterizer, and the encoder completely unchanged. One new dependency — `nalgebra 0.34` — provides the `Perspective3` and `Isometry3` camera math. Everything else (mesh grid construction, face depth sorting, surface morph animation, camera orbit animation) is implemented in-crate using the existing `Tween<P>` infrastructure. The competitive landscape for this niche (offline Rust video generation with declarative 3D surface animation) remains empty — this is still an unsolved problem.

The recommended approach centers on four design decisions that must be locked in before any rendering code is written: (1) a single `Camera` struct in `camera.rs` owns the entire data-to-screen transform chain with no projection logic anywhere else; (2) `CameraState` stores spherical coordinates (`azimuth_deg`, `elevation_deg`, `distance`) and derives `CanTween`, not a Cartesian eye position, so orbit animation produces correct arc-path motion via linear degree interpolation; (3) painter's algorithm (back-to-front centroid sort with backface culling) handles face occlusion — sufficient and correct for the approximately-convex GAM surfaces that are eidos's primary use case; (4) `SurfacePlot` is a self-contained builder that hands `Vec<Primitive>` back to `SceneBuilder`, mirroring exactly how `Axes` works today, with no 3D state leaking into `SceneBuilder`.

The critical risks are all architectural, not algorithmic. Gimbal lock from Euler-angle interpolation, coordinate space proliferation when projection logic is distributed across structs, and SVG polygon count hitting a performance wall at medium mesh resolution are the three failure modes most likely to cause a partial rewrite if not addressed in Phase 1. The painter's algorithm has a known failure mode for non-convex meshes, but GAM surfaces over regular grids are approximately convex at any reasonable viewpoint, making it the correct tradeoff for v1.1. The absence of GPU dependencies, C library linkage, or new rendering paths is a deliberate design strength — it preserves the pure-Rust, cross-compilation-friendly property of the existing pipeline.

## Key Findings

### Recommended Stack

The v1.0 pipeline (`svg 0.18` → `usvg`/`resvg 0.46`/`tiny-skia 0.11` → `ffmpeg-sidecar 2.3`) is untouched for v1.1. The only addition is `nalgebra 0.34` for 3D linear algebra. The choice of nalgebra over glam is decisive: `Perspective3::project_point()` and `Isometry3::look_at_rh()` are first-class types that make perspective projection and look-at camera construction one-liners, whereas glam requires manual matrix math for equivalent results. nalgebra is also f64-native throughout — essential for scientific data precision where GAM output values may span wide ranges. All mesh grid construction, face sorting, colormap interpolation, camera spherical conversion, and surface morph interpolation are implemented in-crate with no additional dependencies.

**Core technologies:**
- `svg 0.18` + `resvg 0.46` + `tiny-skia 0.11`: SVG generation and rasterization — unchanged from v1.0, HIGH confidence
- `ffmpeg-sidecar 2.3`: Frame-to-MP4 encoding via subprocess — unchanged, HIGH confidence
- `simple-easing 1.0`: Easing functions for animation curves — unchanged, HIGH confidence
- `nalgebra 0.34` (new): 3D linear algebra (`Perspective3`, `Isometry3`, f64-native) — single new dependency, HIGH confidence

See `.planning/research/STACK.md` for full rationale, version pinning strategy, and rejected alternatives.

### Expected Features

The v1.1 feature set is scoped to GAM/ML surface presentation. All P1 features reuse the existing `Tween<P>` and `CanTween` infrastructure — no new animation machinery is required. The surface morph (flat→fitted) is the same pattern as SplineFit from v1.0, extended to a 2D vertex grid. Camera orbit animation is a `Tween<CameraState>` where `CameraState` derives `CanTween`. The dependency graph has a clear foundation: perspective projection is a prerequisite for every other 3D feature.

**Must have (table stakes for v1.1 launch):**
- Perspective projection transform — the mathematical foundation; everything else depends on it
- Wireframe mesh rendering — configurable viewpoint, grid density, depth-sorted via painter's algorithm
- Shaded surface with z-height colormap — filled quads with per-face color from z-value gradient
- 3D cartesian axes with tick marks and labels — projected axis lines; correct edge selection
- Flat-to-fitted surface morph animation — per-vertex z-value tween from mean_z to fitted_z
- Orbit camera animation — azimuth sweep via `Tween<CameraState>` with existing Easing variants
- Scatter points in 3D — projected `(x, y, z)` points participating in depth sort with depth-based opacity

**Should have (add after core MVP is validated):**
- Wireframe + filled hybrid mode — single enum option to overlay grid lines on shaded surface
- Elevation swing animation — complement to orbit, same Tween pattern, trivial to add
- Scatter FadeIn animation — scatter appears after surface settles, reuses existing animation infrastructure

**Defer (v2+):**
- Contour projection on base plane — non-trivial to compute z-contours and project correctly; high value but low urgency
- Directional shading / lightsource — brightness multiplier depth cue; useful but not essential
- Unstructured triangle mesh — not needed for regular-grid GAM surfaces
- Interactive rotation — fundamentally incompatible with the video-only, headless architecture
- GPU rendering path — breaks the SVG pipeline; only justified if performance wall cannot be solved at software level

See `.planning/research/FEATURES.md` for the full feature dependency tree, competitor comparison table, and prioritization matrix.

### Architecture Approach

The architecture extends the existing `dataviz` module with three new files (`camera.rs`, `surface_plot.rs`, `scatter_plot3d.rs`) and additive-only modifications to `dataviz/mod.rs`, `scene.rs`, and `lib.rs`. No existing files change in substance. The 3D-to-2D projection lives entirely inside `SurfacePlot::to_primitives(&camera, t_secs)`, returning `Vec<Primitive>` — the same contract as `Axes::to_primitives()`. `svg_gen.rs`, `tiny-skia` rasterization, and the ffmpeg pipeline are untouched. The `Primitive` enum gains no new variants.

**Major components:**
1. `camera.rs` — `Camera` struct, `CameraState` (derives `CanTween`), look-at matrix, `project_to_screen()`, `face_normal()`, `backface_cull()` — the entire transform chain from data space to pixel coordinates
2. `surface_plot.rs` — `SurfacePlot` struct, `to_primitives(&camera, t_secs)` with painter's algorithm face sort + backface culling, z-height colormap, filled quad emission via `Bezier`, fitting animation via `Tween<f64>` progress
3. `scatter_plot3d.rs` — `ScatterPlot3D`, projects `Vec<(f64,f64,f64)>` to depth-sorted `Vec<Circle>` with alpha scaled by view-space depth

**Prescribed build order (dependency-driven):**
1. `camera.rs` — unit-test projection of known world points to expected screen coordinates
2. `SurfacePlot` static rendering — visually validate face sorting and shading before adding time dimension
3. `SceneBuilder::add_surface_plot()` — 5-line convenience method; enables final public API from this point
4. Surface fitting animation — port SplineFit's `animate_fit()` pattern to a 2D vertex grid
5. `ScatterPlot3D` — projection only, no face sorting; battle-tested camera math makes this simple
6. Camera orbit animation — integration test demonstrating `Tween<CameraState>` orbit; no new library code

See `.planning/research/ARCHITECTURE.md` for component table, data flow diagrams, code examples, and five anti-patterns to avoid.

### Critical Pitfalls

1. **Gimbal lock from Euler-angle camera interpolation** — Store `CameraState` in spherical coordinates (`azimuth_deg`, `elevation_deg`, `distance`), not Cartesian eye position. Linear degree interpolation produces correct arc-path orbit motion and avoids the pole-region stutter that Cartesian linear interpolation exhibits. Document the 180° arc limit; full quaternion SLERP is a v1.2 concern only if pole traversal is needed.

2. **Coordinate space proliferation** — All projection logic lives in `camera.rs` and nowhere else. Variables are named explicitly (`p_data`, `p_world`, `p_view`, `p_screen`) so the coordinate space is visible at every call site. Any function with "transform" or "projection" in its name that is not on `Camera` is a warning sign.

3. **SVG polygon count performance wall** — A 30x30 mesh generates 900 quads per frame; at 30fps for 10 seconds that is 270,000 polygon elements to generate, parse, and rasterize. Benchmark per-frame render time at this resolution before adding animation complexity. Apply backface culling before the sort step (halves polygon count). Document 20x20 as the default and maximum recommended resolution for v1.1.

4. **Painter's algorithm failure on non-convex geometry** — Centroid-sort fails silently at camera angles that expose cyclic polygon overlap on non-convex surfaces, producing wrong occlusion with no error signal. For GAM surfaces over regular grids this is acceptable; document the limitation explicitly. Backface culling (Pitfall 5) eliminates most artifacts. BSP tree is the correct fix but is v1.2+ scope.

5. **Surface morph vertex topology mismatch** — The flat and fitted states must have identical vertex count and grid topology. Enforce this structurally: derive both from the same grid, store only z-values as `Vec<f64>` in `SurfaceMeshState`, assert count equality with a clear error message. Two separate constructors with independent `resolution` parameters are the warning sign.

6. **3D API feels grafted on** — Settle the `SurfacePlot` vs `SceneBuilder` boundary before writing any rendering code. `SurfacePlot` owns its camera. `SceneBuilder` never holds a `Camera`. The correct pattern is `builder.add_surface_plot(&plot, &camera, t_secs)`, mirroring `add_axes()`. A `camera: Option<Camera>` field on `SceneBuilder` is the wrong path.

See `.planning/research/PITFALLS.md` for 9 detailed pitfalls with warning signs, recovery strategies, performance traps, and a pitfall-to-phase mapping table.

## Implications for Roadmap

The feature dependency graph prescribes a clear 4-phase order. Each phase has a hard prerequisite on the previous one; no phase can be safely reordered.

### Phase 1: Camera Math and Projection Foundation

**Rationale:** Every other new component depends on the camera and projection math. Isolating and unit-testing this math before building on top of it prevents projection bugs from becoming entangled with surface rendering or animation bugs. This is also where the three highest-severity architectural pitfalls (gimbal lock, coordinate space proliferation, 3D API shape) must be resolved before any surface code exists.

**Delivers:** `camera.rs` with `Camera`, `CameraState` (derives `CanTween`), look-at matrix, `project_to_screen()`, `face_normal()`, `backface_cull()`. Unit tests validating projection of known world points. API shape of `SurfacePlot` and `add_surface_plot()` settled.

**Addresses (from FEATURES.md):** Perspective projection transform — the P1 prerequisite for every other P1 feature.

**Avoids (from PITFALLS.md):** Gimbal lock (#2), coordinate space proliferation (#3), 3D API grafted on (#8).

**Research flag:** Standard — perspective projection math is canonical (Scratchapixel, LearnOpenGL). Skip `/gsd:research-phase`.

### Phase 2: Static 3D Surface Rendering

**Rationale:** Build and visually validate face sorting, backface culling, and shading on a static surface before introducing the time dimension. A confirmed working static render makes animation regression debugging straightforward. The performance baseline must be established here — benchmarking at target mesh resolution before adding animation avoids discovering a performance wall only after the feature is "done."

**Delivers:** `SurfacePlot::new(x_pts, y_pts, z_grid)` with static `to_primitives(&camera, 1.0)`, painter's algorithm with backface culling, z-height colormap, filled quad emission, wireframe mode, `SceneBuilder::add_surface_plot()`. Per-frame render time benchmark at 30x30 mesh.

**Addresses (from FEATURES.md):** Wireframe mesh rendering (P1), shaded surface + z-colormap (P1), configurable viewpoint (P1).

**Avoids (from PITFALLS.md):** Painter's algorithm failure (#1), backface culling skipped (#5), axis labels at wrong SVG depth (#7), SVG performance wall (#4).

**Research flag:** Standard — painter's algorithm and backface culling are fully documented. SVG performance is empirical; benchmark during implementation. Skip `/gsd:research-phase`.

### Phase 3: Surface Fitting and Camera Orbit Animation

**Rationale:** Animation is layered on top of the validated static surface. Both animation types reuse the existing `Tween<P>` infrastructure — no new animation machinery. Surface morph (flat→fitted) is the primary v1.1 differentiator and directly mirrors the SplineFit pattern from v1.0. Camera orbit is the primary mechanism for conveying 3D structure to a passive video viewer.

**Delivers:** `SurfacePlot::animate_fit(start_time, duration, easing)` with flat→fitted z-value tween (progress `Tween<f64>`, per-vertex morph), `Tween<CameraState>` orbit animation driving per-frame reprojection, integration test rendering a morphing surface with orbiting camera to MP4.

**Addresses (from FEATURES.md):** Flat-to-fitted surface morph animation (P1), orbit camera animation (P1), smooth camera easing (P1 — comes for free via existing `Easing` variants).

**Avoids (from PITFALLS.md):** Surface morph vertex topology mismatch (#6), camera orbit pole discontinuity (edge case of #2).

**Research flag:** Standard — `Tween<f64>` morph pattern is a direct port of SplineFit from v1.0. Skip `/gsd:research-phase`.

### Phase 4: Scatter Points and 3D Axes

**Rationale:** Scatter points and 3D axes both depend on projection (Phase 1) and face-sorted rendering (Phase 2) being correct. Scatter points must participate in the depth sort alongside mesh faces — placing them in a post-hoc overlay loop is a named pitfall. 3D axes require deciding which of the 12 bounding-box edges to draw for a given camera orientation, which is the one sub-problem in v1.1 with sparse SVG-specific documentation.

**Delivers:** `ScatterPlot3D::new(points)` with projected scatter circles participating in depth sort alongside mesh faces, depth-based opacity. 3D cartesian axes with projected tick marks and labels; axis line depth-sorted against mesh faces; tick labels always on top.

**Addresses (from FEATURES.md):** Scatter points in 3D (P1), depth-based scatter opacity (P1), 3D cartesian axes (P1), axis labels and tick marks (P1).

**Avoids (from PITFALLS.md):** Scatter Z-depth not in polygon sort (#9), axis labels at wrong SVG depth (#7).

**Research flag:** The heterogeneous depth sort (mesh faces + scatter circles in one list) is standard. Axis bounding-box edge selection for 3D projections has sparse SVG-specific documentation — consider a short research spike before implementing axis rendering, referencing the prideout.net SVG wireframe blog or the TomasHubelbauer/svg-3d reference implementation.

### Phase Ordering Rationale

- Camera math is a strict prerequisite for all 3D features — Phase 1 cannot be reordered
- Static rendering before animation isolates visual bugs from temporal bugs; a working static frame at t=1.0 makes animation regressions obvious
- Animation before scatter/axes keeps each phase's bug surface minimal: projection and face sorting are battle-tested before the complexity of scatter depth integration and axis edge selection is added
- Scatter and 3D axes share the same depth-sort concerns and belong in a single phase to avoid building the heterogeneous sort list twice

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 4 (3D axis edge selection sub-problem):** Which of the 12 bounding-box edges to show as axis lines — and how to determine this from camera orientation without hardcoding angles — has sparse documentation for SVG-based (non-GPU) implementations. Short research spike recommended before writing axis code.

Phases with standard patterns (skip `/gsd:research-phase`):
- **Phase 1:** Perspective projection math is canonical — Scratchapixel and LearnOpenGL cover it completely with verified derivations.
- **Phase 2:** Painter's algorithm and backface culling are fully documented. SVG performance is empirical — measure, don't speculate.
- **Phase 3:** Surface morph is a direct port of existing SplineFit; `Tween<CameraState>` follows the exact same pattern as `Tween<CircleState>` in v1.0.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core pipeline verified against official crates.io/docs.rs; nalgebra 0.34 API confirmed via docs.rs; version 0.34.1 released Sep 20, 2025 — actively maintained |
| Features | MEDIUM-HIGH | Table stakes derived from matplotlib/plotly/rayshader feature analysis with confirmed bug reports; P1 scope is conservative and grounded in the GAM use case; P2/P3 deferral is well-justified |
| Architecture | HIGH | All integration points verified against existing eidos source code; `Axes::to_primitives()`, `SplineFit`, `CanTween` derive patterns are confirmed working and directly applicable |
| Pitfalls | HIGH | Gimbal lock, painter's algorithm failure, coordinate space pitfalls sourced from canonical references (Wikipedia, LearnOpenGL, Scratchapixel, prideout.net); SVG performance limits sourced from benchmark data |

**Overall confidence:** HIGH

### Gaps to Address

- **Quaternion SLERP for pole-region camera orbits:** The v1.1 design uses linear degree interpolation in spherical coordinates, which produces incorrect results when an orbit arc crosses the elevation ±90° pole. The mitigation is to document the 180° arc limit. If any v1.1 example requires polar orbit, quaternion SLERP must be added. Flag during Phase 3 planning if pole traversal is needed.

- **3D axis bounding-box edge selection:** Which edges to draw for a given camera angle requires a non-trivial visibility test. The prideout.net SVG wireframe blog and the TomasHubelbauer/svg-3d reference cover this but for general wireframes, not specifically for labeled plot axes. Validate the approach before Phase 4 implementation begins.

- **SVG per-frame string size at production mesh resolution:** The performance wall is documented but not empirically validated for eidos's specific resvg/tiny-skia pipeline. The first concrete output of Phase 2 must be a per-frame benchmark at 30x30 mesh before any other feature work proceeds. If frame generation exceeds 100ms at this resolution, the mitigation path (combining all face polygons into a single `<path>` element) is defined but requires a significant SVG generation refactor.

- **Scatter-to-face depth sort semantics:** The heterogeneous `Vec<DepthSortedElement>` design for combining mesh faces (centroid Z) and scatter circles (point Z) in a single sort pass is architecturally clean but the sort key semantics differ: a face centroid Z is an average of four vertices, while a scatter point Z is exact. This is not a bug but warrants explicit documentation and testing at Phase 4 — particularly for scatter points at Z values near the surface itself.

## Sources

### Primary (HIGH confidence)

- [resvg GitHub (linebender)](https://github.com/linebender/resvg) — SVG renderer v0.46.0, core pipeline
- [tiny-skia GitHub (linebender)](https://github.com/linebender/tiny-skia) — rasterization backend v0.11.4
- [ffmpeg-sidecar GitHub](https://github.com/nathanbabcock/ffmpeg-sidecar) — subprocess FFmpeg v2.3.0
- [docs.rs nalgebra Perspective3](https://docs.rs/nalgebra/latest/nalgebra/geometry/struct.Perspective3.html) — `project_point()` API confirmed
- [docs.rs nalgebra Isometry3](https://docs.rs/nalgebra/latest/nalgebra/geometry/type.Isometry3.html) — `look_at_rh()` API confirmed
- [3D Wireframes in SVG — Philip Rideout](https://prideout.net/blog/svg_wireframes/) — painter's algorithm for SVG face ordering, backface culling via winding
- [Scratchapixel — Perspective Projection Matrix](https://www.scratchapixel.com/lessons/3d-basic-rendering/perspective-and-orthographic-projection-matrix/building-basic-perspective-projection-matrix.html) — canonical projection math reference
- [Painter's Algorithm — Wikipedia](https://en.wikipedia.org/wiki/Painter%27s_algorithm) — algorithm and cyclic overlap failure cases
- [LearnOpenGL — Coordinate Systems](https://learnopengl.com/Getting-started/Coordinate-Systems) — five-space transform chain
- [Back-Face Culling — Wikipedia](https://en.wikipedia.org/wiki/Back-face_culling) — dot product test, winding order
- [LearnOpenGL — Face Culling](https://learnopengl.com/Advanced-OpenGL/Face-culling) — winding order consistency
- Existing eidos source code — authoritative ground truth for all integration points

### Secondary (MEDIUM confidence)

- [lib.rs/crates/nalgebra](https://lib.rs/crates/nalgebra) — version 0.34.1 (Sep 20, 2025) confirmed
- [dimforge 2025 review](https://dimforge.com/blog/2026/01/09/the-year-2025-in-dimforge/) — nalgebra vs glam design rationale from nalgebra authors
- [matplotlib mplot3d stable docs](https://matplotlib.org/stable/users/explain/toolkits/mplot3d.html) — feature inventory, view_init API
- [Plotly 3D surface plots (Python)](https://plotly.com/python/3d-surface-plots/) — colorscale, contour projection, camera config
- [matplotlib GitHub issues #14148, #23392, #5830](https://github.com/matplotlib/matplotlib) — confirmed zorder and scatter3D depth bugs in painter's-algorithm-based renderer
- [rayshader (tylermorganwall)](https://www.rayshader.com/) — cinematic camera animation patterns for R
- [SLERP — Wikipedia](https://en.wikipedia.org/wiki/Slerp) — constant angular velocity guarantee for quaternion interpolation
- [SVGGenie SVG vs Canvas vs WebGL Performance 2025](https://www.svggenie.com/blog/svg-vs-canvas-vs-webgl-performance-2025) — SVG DOM overhead limits above 5000 nodes
- [Gimbal Lock — Wikipedia](https://en.wikipedia.org/wiki/Gimbal_lock) — root cause analysis

### Tertiary (LOW confidence)

- [SVG 3D projection — TomasHubelbauer/svg-3d](https://github.com/TomasHubelbauer/svg-3d) — reference implementation for SVG polygon depth sorting and axis edge selection
- [Effective Multi-Dimensional 3D Scatterplots](https://arxiv.org/html/2406.06146v2) — depth-based opacity patterns for scatter points
- [Quaternion Rotation — Ralf Becker](https://medium.com/@ratwolf/quaternion-3d-rotation-32a3de61a373) — construction-vs-storage distinction for gimbal lock

---
*Research completed: 2026-02-25*
*Ready for roadmap: yes*
