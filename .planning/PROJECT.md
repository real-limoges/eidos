# eidos

## What This Is

A Manim-inspired Rust library for programmatic data visualization and animation. Users describe scenes declaratively — what objects appear, not when or how — and eidos handles animation, interpolation, and rendering to video. Built initially for GAM visualizations (spline fits, partial dependence plots, confidence bands), extended in v1.1 with full 3D surface rendering: perspective mesh (wireframe/shaded), surface morph animation, camera orbit animation, and depth-sorted scatter points.

v1.0 shipped with full rendering pipeline, animation engine, cartesian data visualization, and GAM-specific primitives. v1.1 added 3D perspective surface visualization with camera, colormap, axes, and scatter points.

## Core Value

A Rust-native way to produce beautiful, animated data visualizations with a declarative API — no Python, no GUI, just code that describes a scene and produces a video.

## Current State (after v1.1)

- **Codebase:** ~5,375 lines Rust (src/)
- **Tech stack:** Rust 2024 · nalgebra 0.34 · tiny-skia/resvg · ffmpeg
- **Test suite:** 151 tests (119 lib + 32 integration) · 0 failures
- **Test binaries:** integration, data_viz, gam_viz, scatter_points, surface_animation, surface_rendering

**New in v1.1:**
- `Camera` — spherical-coordinate perspective projection with Y-flip SVG mapping
- `SurfacePlot` — 3D surface from `Vec<f64>` grids, three render modes (Shaded/Wireframe/ShadedWireframe)
- `draw_axes()` — 3D cartesian axes with quadrant-aware edge selection, tick marks, tick labels
- `viridis_color()` — 256-entry viridis colormap LUT for z-height face coloring
- `animate_fit()` / `to_primitives_at()` — surface morphing from flat to fitted shape
- `animate_camera_azimuth()` / `camera_at()` — camera orbit animation over time
- `ScatterPlot` — raw `(x,y,z)` points projected to depth-sorted circles with exponential opacity falloff and `animate_fade()`
- `SceneBuilder::add_surface()`, `add_surface_at()`, `add_scatter()`, `add_scatter_at()`, `merge_scatter()`

## Requirements

### Validated

- ✓ User can render a scene to an MP4 video file — v1.0 (SVG→rasterize→ffmpeg pipeline)
- ✓ User can configure video resolution and framerate — v1.0
- ✓ User can add a circle, rectangle, line, arrow, text label, bezier curve with configurable styling — v1.0
- ✓ User can animate any visual property between two states with easing functions — v1.0 (Tween, 4 Easing variants)
- ✓ User can compose multiple animations in parallel — v1.0
- ✓ User can create 2D cartesian axes with ticks, labels, and configurable range — v1.0 (Heckbert algorithm)
- ✓ User can construct a smooth curve from Vec<(f64, f64)> data points — v1.0 (Catmull-Rom splines)
- ✓ Axes auto-range to fit provided data — v1.0
- ✓ User can create a confidence band between two curves — v1.0 (ConfidenceBand)
- ✓ User can animate a spline fitting to data — v1.0 (SplineFit with frame-time morphing)
- ✓ User can create a 3D surface plot from a regular grid of (x, y, z) data with a configurable camera viewpoint — v1.1 (SURF-01)
- ✓ User can render the surface as a wireframe mesh (depth-sorted projected edges) — v1.1 (SURF-02)
- ✓ User can render the surface as a shaded mesh with a z-height color gradient — v1.1 (SURF-03)
- ✓ User can add 3D cartesian axes with projected tick marks and labels to a surface plot — v1.1 (SURF-04)
- ✓ User can add (x, y, z) scatter points to a 3D plot, rendered with depth-based opacity — v1.1 (SCAT-01)
- ✓ User can animate scatter points fading in over a specified time range — v1.1 (SCAT-02)
- ✓ User can animate the surface morphing from flat to fitted shape over a specified time range — v1.1 (ANIM-01)
- ✓ User can animate the camera orbiting around the surface (azimuth sweep) over a specified time range — v1.1 (ANIM-02)

### Active

<!-- v1.2 — API Polish & Ergonomics -->

- [ ] State types (`CircleState`, `RectState`, `LineState`, `TextState`) accept `Color` directly — no raw `fill_r/g/b` f64 fields required (ERGO-01)
- [ ] `Tween` fluent builder API — chain `.from()/.to()/.start_at()/.over()/.easing()` instead of struct literal (ERGO-02)
- [ ] `Axes::map_point(data_x, data_y) -> (f64, f64)` — data-to-pixel coordinate helper (COORD-01)
- [ ] All primitive builder methods infallible — `.opacity()`, `.stroke()`, `.font_size()` return `Self` with clamped values (API-01)

### Out of Scope

- GUI or interactive editor — code-only, by design
- LaTeX rendering — not needed for GAM visualization focus
- Real-time/interactive output — video files only
- Python bindings — the whole point is to stay in Rust
- Interactive rotation — requires event loop; fundamentally incompatible with video-only headless architecture
- GPU rendering path — breaks the SVG pipeline; only justified if software performance wall cannot be solved

## Deferred to v2+

- **REND-01**: Contour projection onto the base plane
- **REND-02**: Directional lighting / Lambertian shading
- **REND-03**: Unstructured triangle mesh input (non-regular grids)
- **CAM-01**: Full quaternion SLERP camera interpolation for pole-region orbits (>180° arcs)
- **CAM-02**: Elevation swing animation as complement to azimuth orbit

## Constraints

- **Language**: Rust — no Python, no FFI to Python ecosystem
- **Output**: Video files (MP4) — not interactive, not browser-based
- **API**: Declarative builder — `SceneBuilder`, not imperative sequencing

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| SVG → rasterize → ffmpeg pipeline | Clean vector look, composable, no GPU dependency | ✓ Good — clean output, no GPU needed |
| Declarative "describe scene, library animates" API | Matches user mental model, differentiates from raw drawing libs | ✓ Good — natural to use |
| GAM primitives as first-class objects | Drives initial design with concrete use cases before generalizing | ✓ Good — ConfidenceBand + SplineFit delivered |
| EidosError with two variants (InvalidConfig, RenderFailed) | Opaque to callers, covers distinct failure modes | ✓ Good — minimal and sufficient |
| Color uses u8 RGB components | Sufficient for SVG display, avoids f32 ergonomics issues | ✓ Good |
| Primitive enum with From impls (not trait objects) | Enum dispatch avoids dyn overhead, simpler match in svg_gen | ✓ Good — clean dispatch |
| fontdb as Arc<fontdb::Database> | resvg 0.47 requires Arc; cheap clone for multi-frame use | ✓ Good |
| State structs use f64 channels (0..=255) | No overflow during interpolation; cast to u8 only at to_*() | ✓ Good |
| plot_bounds() duplicates to_primitives() Steps 1-2 | Avoids refactoring production rendering path | ⚠️ Revisit — mild duplication |
| All user-facing types re-exported at crate root | Ergonomic imports: `use eidos::{SceneBuilder, Circle, CircleState, ...}` | ✓ Good |
| encode_to_mp4 deleted entirely (v1.0) | Zero callers confirmed; full removal cleaner than permanent deprecation | ✓ Good |
| Camera owns entire data-to-screen transform chain | No projection logic outside camera.rs | ✓ Good — clean separation |
| CameraState uses spherical coordinates (azimuth, elevation, distance) | Avoids gimbal lock, enables clean Tween<CameraState> orbit | ✓ Good |
| Painter's algorithm for face occlusion | Correct for approximately-convex GAM surfaces, simple to implement | ✓ Good — sub-millisecond for 30×30 mesh |
| SurfacePlot is self-contained; SceneBuilder never holds Camera | Mirrors 2D Axes pattern, avoids coupling | ✓ Good |
| fitted_zs snapshot at SurfacePlot::new() | Immutable; enables to_primitives_at(&self) safe for Fn closures | ✓ Good |
| z_at() hold semantics: before=0.0, gap=fitted_z, after=fitted_z | Natural "appear then hold" UX for surface morph | ✓ Good |
| camera_at() returns None with no animations | Caller uses static camera; clean zero-cost path | ✓ Good |
| far_floor_corner() uses integer quadrant cast | Avoids float edge cases at 90°/180°/270°/360° boundaries | ✓ Good |
| SceneBuilder carries prim_depths Vec<f64> parallel to primitives | Enables O(n+m) merge of scatter circles with surface faces | ✓ Good |
| Non-surface primitives (axes, labels) get prim_depths = NEG_INFINITY | Always painted on top; never occluded by surface or scatter | ✓ Good |
| requirements-completed YAML key uses hyphens (not underscores) | gsd-tools reads fm['requirements-completed'] per commands.cjs:307 | ✓ Documented |

## Current Milestone: v1.2 API Polish & Ergonomics

**Goal:** Eliminate the most common friction points in the eidos API — verbose animation state construction, manual coordinate math, and unpredictable `?` in builder chains.

**Target features:**
- State types accept `Color` directly
- `Tween` fluent builder
- `Axes::map_point()` coordinate helper
- Infallible primitive builders

---
*Last updated: 2026-02-26 after v1.2 milestone start — API Polish & Ergonomics*
