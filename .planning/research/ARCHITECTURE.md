# Architecture Research

**Domain:** 3D Surface Rendering Integration into 2D SVG Animation Pipeline
**Researched:** 2026-02-25
**Confidence:** HIGH

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        USER API LAYER                               │
│  ┌──────────────┐  ┌───────────────┐  ┌──────────────────────────┐  │
│  │  SurfacePlot │  │  Camera /     │  │  ScatterPlot3D           │  │
│  │  (NEW)       │  │  CameraState  │  │  (NEW)                   │  │
│  │              │  │  (NEW)        │  │                          │  │
│  └──────┬───────┘  └──────┬────────┘  └────────────┬─────────────┘  │
│         │                 │                        │               │
├─────────┴─────────────────┴────────────────────────┴───────────────┤
│                   PROJECTION LAYER (new, dataviz module)            │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  camera.rs: project(point3d, camera) → (screen_x, screen_y)  │  │
│  │             sort_faces_back_to_front(faces)                   │  │
│  │             backface_cull(normal, view_dir) → bool            │  │
│  └───────────────────────────┬───────────────────────────────────┘  │
│                              │                                      │
├──────────────────────────────┴─────────────────────────────────────┤
│               EXISTING PRIMITIVES / SVG PATH LAYER (unchanged)      │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐   │
│  │ Bezier   │  │  Line    │  │  Circle  │  │  Rect / Text     │   │
│  │ (filled) │  │          │  │  (dots)  │  │                  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘   │
├─────────────────────────────────────────────────────────────────────┤
│               EXISTING RENDERING PIPELINE (unchanged)               │
│     SVG per frame → tiny-skia/resvg rasterize → ffmpeg MP4         │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | New vs Existing |
|-----------|----------------|-----------------|
| `SurfacePlot` | Holds 3D grid data, surface color config, fitting animation state | NEW — dataviz module |
| `Camera` | Holds spherical viewpoint (azimuth, elevation, distance, target) + viewport dims | NEW — dataviz module |
| `CameraState` | All-f64 CanTween struct mirroring Camera's spherical fields | NEW — animation compatible |
| `ScatterPlot3D` | Holds `Vec<(f64,f64,f64)>` raw data points for scatter overlay | NEW — dataviz module |
| `Projection3D` (functions in camera.rs) | Pure math: world → view → clip → screen; no SVG knowledge | NEW — inside camera.rs |
| Face sorting | Sort mesh quads by Z-centroid; painter's algorithm | NEW — inside surface_plot.rs |
| `Bezier` (filled) | Already supports `.fill(Color)` — used for shaded quad faces | EXISTING — no change needed |
| `Circle` | Already exists — used for projected data point scatter dots | EXISTING — no change needed |
| `SceneBuilder` | Gains `add_surface_plot()` convenience method | MODIFIED — adds one method |
| `lib.rs` | Gains re-exports for Camera, CameraState, SurfacePlot, ScatterPlot3D | MODIFIED — additive only |
| `dataviz/mod.rs` | Gains module declarations and re-exports for new types | MODIFIED — additive only |
| `Primitive` enum | Unchanged — no new variants needed | EXISTING — no change |
| `svg_gen.rs` | Unchanged — only ever receives existing Primitive types | EXISTING — no change |
| `Tween<P>` | Unchanged — works as-is; `Tween<CameraState>` and `Tween<f64>` both used | EXISTING — no change |

## Recommended Project Structure

```
src/
├── primitives/            # unchanged
│   ├── bezier.rs          # unchanged — filled Bezier already supports quads
│   ├── circle.rs          # unchanged — used for scatter dots
│   └── mod.rs             # unchanged
├── animation/             # unchanged
│   ├── tween.rs           # unchanged — CameraState derives CanTween same as CircleState
│   └── easing.rs          # unchanged
├── dataviz/
│   ├── mod.rs             # MODIFIED: add re-exports for SurfacePlot, Camera, CameraState, ScatterPlot3D
│   ├── axes.rs            # unchanged
│   ├── data_curve.rs      # unchanged
│   ├── confidence_band.rs # unchanged
│   ├── spline_fit.rs      # unchanged — 2D SplineFit stays 2D
│   ├── spline.rs          # unchanged
│   ├── camera.rs          # NEW: Camera struct, CameraState (CanTween), projection math functions
│   ├── surface_plot.rs    # NEW: SurfacePlot struct + to_primitives(&camera, t_secs)
│   └── scatter_plot3d.rs  # NEW: ScatterPlot3D struct + to_primitives(&camera)
├── scene.rs               # MODIFIED: add add_surface_plot() to SceneBuilder
├── svg_gen.rs             # unchanged
└── lib.rs                 # MODIFIED: add re-exports for new public types
```

### Structure Rationale

- **camera.rs:** Camera struct and projection math are co-located because they are tightly coupled — every projection call needs camera parameters. Separating them would require passing large parameter bags.
- **surface_plot.rs:** SurfacePlot owns its decomposition. It calls into camera.rs for projection, sorts faces internally, and returns `Vec<Primitive>`. This mirrors the Axes pattern exactly — high-level dataviz type that decomposes to primitives.
- **scatter_plot3d.rs:** Separate file mirrors the DataCurve/ConfidenceBand pattern. Small composable dataviz types that do one thing each.
- **No new rendering modules:** The SVG pipeline is untouched. 3D→2D projection is a data transformation inside the dataviz layer before `Vec<Primitive>` is handed downstream.

## Architectural Patterns

### Pattern 1: 3D→2D Projection as High-Level Primitive Decomposition

**What:** `SurfacePlot::to_primitives(&camera, t_secs)` applies projection internally and returns `Vec<Primitive>` to the caller. The SVG renderer never sees 3D coordinates — it only receives Bezier, Circle, Line, etc. The 3D-to-2D transformation happens entirely within the dataviz layer.

**When to use:** Always, for this codebase. This is the only approach consistent with the existing architecture where Axes, DataCurve, SplineFit all decompose to `Vec<Primitive>`.

**Trade-offs:**
- Pro: Zero changes to svg_gen.rs, rasterization, or ffmpeg pipeline
- Pro: Follows the established decomposition contract (same as Axes::to_primitives)
- Pro: Projection bugs are isolated in one place, not scattered in the renderer
- Con: Cannot add SVG-level features like `<filter>` depth-of-field — not needed for this use case

**Example:**
```rust
// Inside scene render closure — mirrors how Axes is used today
let camera = cam_tween.value_at(t_secs).to_camera();
for p in surface_plot.to_primitives(&camera, t_secs) {
    builder.add(p);
}
// Or via SceneBuilder convenience method:
builder.add_surface_plot(&surface_plot, &camera, t_secs);
```

### Pattern 2: CameraState as CanTween Struct (Camera Animation)

**What:** Camera animation follows the identical pattern as CircleState→Circle. A `CameraState` struct holds all camera parameters as f64 fields in spherical coordinates (azimuth_deg, elevation_deg, distance, target_x, target_y, target_z). It derives `CanTween`. A `to_camera()` method converts interpolated state back to a `Camera` with a Cartesian eye position.

**When to use:** Any camera motion — orbit, zoom, pan — all driven by existing `Tween<CameraState>`.

**Trade-offs:**
- Pro: Reuses entire existing animation infrastructure — no new animation primitives
- Pro: Works with all four existing Easing variants
- Pro: Spherical interpolation (azimuth, elevation) is correct for orbit motion. Linear interpolation in degree-space is geometrically correct for smooth arcs.
- Con: Azimuth interpolation across the 0°/360° boundary is wrong (e.g., 350°→10° goes the long way around). Document that users should keep arcs below 180°; can be fixed in v1.2 with angle normalization.
- Con: Interpolating Cartesian eye position directly (the alternative) does not produce arc motion — it cuts through the interior of the orbit sphere.

**Example:**
```rust
#[derive(Clone, CanTween)]
pub struct CameraState {
    pub azimuth_deg:   f64,  // horizontal orbit angle around target
    pub elevation_deg: f64,  // vertical orbit angle (0=horizon, 90=top-down)
    pub distance:      f64,  // distance from target
    pub target_x:      f64,  // look-at point x
    pub target_y:      f64,
    pub target_z:      f64,
}

impl CameraState {
    pub fn to_camera(&self, viewport_w: u32, viewport_h: u32) -> Camera {
        // Convert spherical (azimuth, elevation, distance) to Cartesian eye position
        let eye_x = self.target_x + self.distance
            * self.elevation_deg.to_radians().cos()
            * self.azimuth_deg.to_radians().sin();
        let eye_y = self.target_y + self.distance
            * self.elevation_deg.to_radians().sin();
        let eye_z = self.target_z + self.distance
            * self.elevation_deg.to_radians().cos()
            * self.azimuth_deg.to_radians().cos();
        Camera { eye: (eye_x, eye_y, eye_z), target: (self.target_x, self.target_y, self.target_z),
                 viewport_w, viewport_h, fov_deg: 45.0 }
    }
}

// Animation — identical to CircleState usage:
let cam_tween: Tween<CameraState> = Tween {
    start: CameraState { azimuth_deg: 30.0, elevation_deg: 30.0, distance: 5.0,
                          target_x: 0.0, target_y: 0.0, target_z: 0.0 },
    end:   CameraState { azimuth_deg: 150.0, ..start },
    start_time: 1.0,
    duration: 4.0,
    easing: Easing::EaseInOut,
};
```

### Pattern 3: Painter's Algorithm for Depth-Sorted SVG Faces

**What:** SurfacePlot produces mesh quads (one quad per grid cell). Each face is projected to screen coordinates and its Z-centroid is computed in view space. Faces are sorted back-to-front by Z-centroid (painter's algorithm). Back-facing quads are culled (dot product of face normal with view direction < 0). Sorted front-facing faces are emitted as filled `Bezier` quads. SVG document order provides occlusion — later-drawn elements paint over earlier ones.

**When to use:** For wireframe and shaded surface rendering where the mesh is approximately convex (typical for GAM surfaces over a rectangular grid). This is the standard technique for 3D→SVG rendering without a depth buffer.

**Trade-offs:**
- Pro: No new rendering path — works entirely through existing Bezier primitives
- Pro: Painter's algorithm is simple: sort faces by Z-centroid, emit back-to-front
- Pro: Verified approach for SVG-based 3D rendering (see prideout.net reference)
- Con: O(n log n) sort per frame on face count. For a 20x20 grid = 361 faces — negligible.
- Con: Painter's algorithm fails for non-convex or self-intersecting meshes. Acceptable for smooth GAM surfaces.

**Example:**
```rust
// Inside SurfacePlot::to_primitives():
let mut faces: Vec<(f64, [ScreenPt; 4], Color)> = grid_quads
    .iter()
    .filter_map(|quad| {
        let normal = face_normal(&quad.world_pts);
        let view_dir = camera.eye_direction_to_target();
        if normal.dot(view_dir) >= 0.0 { return None; } // backface cull
        let projected: Vec<ScreenPt> = quad.world_pts.iter()
            .map(|p| project_to_screen(p, &camera))
            .collect();
        let z_centroid = projected.iter().map(|p| p.z_view).sum::<f64>() / 4.0;
        let shade = compute_diffuse_shade(&normal, &camera, base_color);
        Some((z_centroid, [projected[0], projected[1], projected[2], projected[3]], shade))
    })
    .collect();

// Sort back to front (smallest z_centroid first in view space, farthest first)
faces.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

for (_, pts, color) in &faces {
    let bezier = Bezier::new()
        .move_to(pts[0].x, pts[0].y)
        .line_to(pts[1].x, pts[1].y)
        .line_to(pts[2].x, pts[2].y)
        .line_to(pts[3].x, pts[3].y)
        .close()
        .fill(*color)
        .stroke(Color::rgb(40, 40, 40), 0.5).unwrap();
    prims.push(bezier.into());
}
```

### Pattern 4: Surface Fitting Animation (Flat→Final Morph)

**What:** SurfacePlot holds a `Vec<Vec<f64>>` of fitted z-values. A single `Tween<f64>` progress in [0.0, 1.0] drives per-vertex interpolation from a flat surface (all z = mean_z) to the final fitted surface. This is the SplineFit animate_fit() pattern extended to a 2D grid.

**When to use:** For animated surface reveal. Progress is a `Tween<f64>` using existing animation infrastructure — no new animation types needed.

**Trade-offs:**
- Pro: Reuses exact strategy from SplineFit (progress Tween, morph formula, then project)
- Pro: All vertices morph simultaneously — correct behavior for 3D surfaces (unlike SplineFit's left-to-right reveal)
- Pro: Zero new animation infrastructure

**Example:**
```rust
// Inside SurfacePlot::to_primitives():
let progress: f64 = match &self.animation {
    None => 1.0,
    Some(anim) => {
        let tween = Tween { start: 0.0_f64, end: 1.0_f64,
                            start_time: anim.start_time, duration: anim.duration,
                            easing: anim.easing };
        tween.value_at(t_secs)
    }
};
let mean_z: f64 = self.z_values.iter().flatten().sum::<f64>()
    / (self.z_values.len() * self.z_values[0].len()) as f64;
// Per vertex:
let morphed_z = mean_z + progress * (fitted_z - mean_z);
```

## Data Flow

### Frame Render Flow (3D Surface)

```
Scene::render() calls build_scene(&mut builder, t_secs)
    |
    v
User closure evaluates Tween<CameraState>.value_at(t_secs) → CameraState
    |
    v
CameraState.to_camera(width, height) → Camera { eye, target, up, fov, viewport }
    |
    v
SurfacePlot.to_primitives(&camera, t_secs)
    |
    v  [inside to_primitives]:
    1. compute progress = Tween<f64>.value_at(t_secs)  ← fitting animation
    2. interpolate z-values: z = mean_z + progress * (fitted_z - mean_z)
    3. for each grid quad: project 4 world corners → screen (px, py) + view-space z_centroid
    4. backface cull: skip if face_normal · view_dir >= 0
    5. sort faces by z_centroid ascending (farthest first = back to front in SVG)
    6. emit each face as filled Bezier with diffuse shading (color from z-height + light dir)
    7. optionally emit wireframe lines on top
    |
    v
ScatterPlot3D.to_primitives(&camera) → Vec<Circle>  (projected scatter dots)
    |
    v
Vec<Primitive>  (all Bezier, Circle — no new Primitive variants)
    |
    v
SceneBuilder.primitives accumulates all
    |
    v
svg_gen::build_svg_document()   ← UNCHANGED
    |
    v
svg_gen::rasterize_frame()      ← UNCHANGED
    |
    v
ffmpeg stdin                     ← UNCHANGED
```

### Camera Orbit Animation Flow

```
Tween<CameraState> { start, end, start_time, duration, easing }
    | value_at(t_secs)
    v
CameraState { azimuth_deg=lerp, elevation_deg=lerp, distance=lerp, ... }
    | to_camera(viewport_w, viewport_h)
    v
Camera { eye=(x,y,z) from spherical conversion, target, up=(0,1,0), fov_deg=45 }
    | passed to SurfacePlot.to_primitives()
    v
Projection applied per vertex each frame
```

### Projection Math Flow (per vertex)

```
World point (wx, wy, wz)
    | view matrix (look-at transform from camera eye/target/up)
    v
View point (vx, vy, vz)     ← z_centroid for depth sorting taken here (in view space)
    | perspective divide: clip_x = vx / -vz,  clip_y = vy / -vz
    | scale by focal length: f = 1 / tan(fov/2)
    v
Clip space (cx, cy) in [-1, 1]
    | viewport transform
    v
Screen point (px, py) in pixel coordinates  ← what Bezier/Circle receive
```

## Integration Points

### New vs Modified Components

| Component | Status | Change Required |
|-----------|--------|-----------------|
| `svg_gen.rs` | UNCHANGED | None — only ever receives existing Primitive types |
| `Bezier` primitive | UNCHANGED | None — `.fill()` already works for shaded quads |
| `Circle` primitive | UNCHANGED | None — used as-is for projected scatter dots |
| `Line` primitive | UNCHANGED | None — optionally used for axis frame edges |
| `Primitive` enum | UNCHANGED | No new variants needed |
| `Tween<P>` | UNCHANGED | Works as-is; `Tween<CameraState>` and `Tween<f64>` both work |
| `Easing` | UNCHANGED | Works as-is |
| `SceneBuilder` | MODIFIED | Add `add_surface_plot(&SurfacePlot, &Camera, t_secs)` — mirrors `add_axes()` |
| `lib.rs` | MODIFIED | Add re-exports for Camera, CameraState, SurfacePlot, ScatterPlot3D |
| `dataviz/mod.rs` | MODIFIED | Add module declarations and re-exports |
| `camera.rs` | NEW | Camera, CameraState (CanTween), look-at matrix, projection math |
| `surface_plot.rs` | NEW | SurfacePlot struct, to_primitives(), fitting animation |
| `scatter_plot3d.rs` | NEW | ScatterPlot3D, projects Vec<(f64,f64,f64)> to Vec<Circle> |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `SurfacePlot` ↔ `camera.rs` | Direct function calls: `project_to_screen(pt, cam)`, `face_normal(pts)` | Not trait-based; no dynamic dispatch needed |
| `SurfacePlot` → `Primitive` enum | Returns `Vec<Primitive>` — same contract as Axes::to_primitives() | Enables reuse of entire downstream pipeline unchanged |
| `CameraState` ↔ `Tween<P>` | CameraState derives CanTween — identical to CircleState | No new animation infrastructure |
| `SceneBuilder` ↔ `SurfacePlot` | `add_surface_plot()` calls `to_primitives()` and loops `add()` | 5-line method, mirrors `add_axes()` exactly |
| User API ↔ `dataviz` | New types re-exported at crate root | Same ergonomics as v1.0 public types |

## Suggested Build Order

Dependencies flow top-to-bottom. Each step builds only on what exists above it.

```
Step 1: camera.rs
        What: Camera struct, CameraState (#[derive(CanTween)]), look-at matrix,
              project_to_screen() function, face_normal(), backface_cull()
        Depends on: keyframe-derive (existing), f64 math only
        Validates: Unit-test projection with known camera → known expected screen coords
        Note: No surface needed to test projection math in isolation

Step 2: SurfacePlot static rendering (no animation yet)
        What: SurfacePlot::new(x_pts, y_pts, z_grid), to_primitives(&camera, 1.0)
              painter's algorithm face sort, diffuse shading, quad→Bezier
        Depends on: camera.rs (Step 1), Bezier (existing), Primitive enum (existing)
        Validates: Render a static surface, visually inspect output
        Note: Build and validate projection pipeline before adding animation complexity

Step 3: SceneBuilder::add_surface_plot()
        What: One convenience method mirroring add_axes()
        Depends on: surface_plot.rs (Step 2), SceneBuilder (existing)
        Note: 5-line addition; enables all subsequent examples to use final API

Step 4: Surface fitting animation
        What: SurfacePlot::animate_fit(start_time, duration, easing)
              to_primitives() consumes t_secs, evaluates Tween<f64> progress,
              morphs z-values from mean_z to fitted_z
        Depends on: surface_plot.rs (Step 2), Tween<f64> (existing)
        Note: Port SplineFit's animate_fit() pattern to 2D grid; same morph formula

Step 5: ScatterPlot3D
        What: ScatterPlot3D::new(points: Vec<(f64,f64,f64)>),
              to_primitives(&camera) → Vec<Circle> (projected + sized by depth)
        Depends on: camera.rs (Step 1), Circle (existing)
        Note: Simplest new type; only needs projection, no face sorting

Step 6: Camera rotation animation (integration test + docs)
        What: Tween<CameraState> in a render closure, verify orbit looks correct
        Depends on: camera.rs (Step 1), CameraState derives CanTween already
        Note: No new library code needed. This step is integration test + API documentation.
```

### Rationale for This Order

- **Step 1 first:** Every other new component depends on the camera/projection math. Isolating and validating this math before building on top of it prevents debugging projection bugs mixed with surface or animation bugs.
- **Step 2 static before Step 4 animated:** Static rendering validates face sorting and shading visually without the time dimension. Adding animation on top of a verified static render is far safer.
- **Step 3 early:** The SceneBuilder method is trivial but having it early means all subsequent testing uses the final public API.
- **Step 5 after Step 2:** ScatterPlot3D is simple (no face sorting) but projection must be verified first. Building in Step 5 means the projection math is battle-tested.
- **Step 6 as test/doc:** CameraState Tween works as soon as Step 1 is complete. No library code needed — just an integration test demonstrating the orbit animation and documenting the azimuth boundary issue.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Small grid (10x10 = 81 quads) | Per-frame projection + sort is well under 1ms. No changes needed. |
| Medium grid (30x30 = 841 quads) | Still fast; sort is O(n log n) on ~841 elements. Recommended default cap. |
| Large grid (100x100 = 9801 quads) | Sort begins to matter at 30fps. Consider caching sorted order when camera is static. Out of scope for v1.1. |

### Scaling Priorities

1. **First bottleneck:** SVG document size. Each quad becomes a `<path>` element. A 30x30 mesh = 841 path elements per frame. At 30fps x 10s = 300 frames, resvg parses 300 SVG documents with 841 paths each. Profiling needed if render time is unacceptable. Mitigation: cap default grid resolution, expose `resolution` parameter.
2. **Second bottleneck:** Projection computation. 30x30 = 900 vertices projected per frame. Pure f64 math, no allocations — negligible. Only relevant above ~50k vertices.

## Anti-Patterns

### Anti-Pattern 1: Adding a 3D Rendering Path to svg_gen.rs

**What people do:** Add `Primitive::SurfaceMesh(SurfaceMesh)` to the Primitive enum and handle 3D projection inside `build_svg_document()`.

**Why it's wrong:** This entangles projection math (camera, matrices, depth sorting) with the SVG serialization layer. The existing design deliberately keeps svg_gen.rs as a pure Primitive→SVG serializer. Breaking this contract propagates 3D complexity into the wrong layer and makes the rendering pipeline harder to reason about.

**Do this instead:** Project to 2D inside `SurfacePlot::to_primitives()`, return `Vec<Primitive>`, let svg_gen.rs remain completely unchanged.

### Anti-Pattern 2: Deriving CanTween on Camera Directly (Cartesian Interpolation)

**What people do:** Define Camera with Cartesian eye position (eye_x, eye_y, eye_z), derive CanTween directly on Camera, and use `Tween<Camera>` for orbit animation.

**Why it's wrong:** Linear interpolation between two Cartesian eye positions does not follow an arc — it cuts through the interior of the orbit sphere, producing a zoom-in/zoom-out motion rather than a rotation. Spherical parameters (azimuth, elevation, distance) are the correct representation for orbit motion because they interpolate linearly in angular space.

**Do this instead:** Keep CameraState in spherical coordinates. Derive CanTween on CameraState. Convert to Cartesian eye position only at render time via `to_camera()`.

### Anti-Pattern 3: Per-Edge Lines for Wireframe Rendering

**What people do:** Emit each wireframe edge as a `Line` primitive and try to manage visibility by tracking which edges belong to front-facing quads.

**Why it's wrong:** A 20x20 mesh has 760+ grid edges. With per-edge Lines, z-ordering is impossible (Lines have no fill to occlude behind them). Front-face edge detection requires knowing which faces share each edge — O(n²) neighbor lookups. The approach also does not generalize to shaded surfaces.

**Do this instead:** Emit filled `Bezier` quads sorted back-to-front. Use a thin stroke on each filled quad for the wireframe appearance. The fill handles occlusion naturally. The stroke creates the wireframe look without any edge-tracking complexity.

### Anti-Pattern 4: Pre-Computing Projection Outside the Render Closure

**What people do:** Project all mesh vertices to screen space before the render loop starts and cache screen-space coordinates in the SurfacePlot struct.

**Why it's wrong:** Camera animation requires re-projection every frame as the viewpoint changes. Pre-computing screen coordinates assumes a static camera. Even for static cameras, it creates an inconsistent two-phase API where users must call a "project" step before rendering.

**Do this instead:** Project inside `to_primitives(&camera, t_secs)`, which is called per-frame inside the render closure with the current frame's camera state. This is what SplineFit::to_bezier() does — it takes t_secs and recomputes each frame. Consistency with existing patterns matters more than micro-optimization here.

### Anti-Pattern 5: Sorting SVG Elements via Z-Buffer Rather Than Document Order

**What people do:** Attempt to implement a z-buffer by tracking pixel-level coverage and emitting only visible face fragments.

**Why it's wrong:** SVG is not a rasterization API. Per-pixel z-tests require a pixel buffer, not a vector document. Implementing this in SVG would require tessellating faces into non-overlapping regions — complexity that is completely unnecessary for smooth convex surfaces.

**Do this instead:** Painter's algorithm (back-to-front sort by face Z-centroid). For typical smooth GAM surfaces, painter's algorithm is correct and produces no visual artifacts.

## Sources

- Painter's algorithm for SVG face ordering: [3D Wireframes in SVG — Philip Rideout](https://prideout.net/blog/svg_wireframes/) — HIGH confidence (technical blog with verified implementation)
- Perspective projection matrix math: [Scratchapixel — Perspective and Orthographic Projection Matrix](https://www.scratchapixel.com/lessons/3d-basic-rendering/perspective-and-orthographic-projection-matrix/building-basic-perspective-projection-matrix.html) — HIGH confidence (canonical reference)
- Painter's algorithm overview: [Painter's Algorithm — Wikipedia](https://en.wikipedia.org/wiki/Painter%27s_algorithm) — HIGH confidence
- CanTween derive on f64 structs: [keyframe crate — docs.rs](https://docs.rs/keyframe/latest/keyframe/) — HIGH confidence (verified against existing codebase usage in circle.rs)
- 3D projection pipeline: [3D projection — Wikipedia](https://en.wikipedia.org/wiki/3D_projection) — MEDIUM confidence (general reference)
- SVG polygon depth-sorting implementation: [SVG 3D projection — TomasHubelbauer/svg-3d](https://github.com/TomasHubelbauer/svg-3d) — MEDIUM confidence (reference implementation)
- Existing eidos source code (authoritative ground truth for all integration points): HIGH confidence

---
*Architecture research for: eidos v1.1 — 3D Surface Visualization integration into 2D SVG pipeline*
*Researched: 2026-02-25*
