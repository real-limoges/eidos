# Pitfalls Research

**Domain:** Adding 3D surface rendering to an existing 2D SVG-based Rust animation library
**Project:** eidos v1.1 -- 3D Surface Visualization milestone
**Researched:** 2026-02-25
**Confidence:** HIGH

---

## Critical Pitfalls

### Pitfall 1: Painter's Algorithm Failure on Non-Convex Mesh Geometry

**What goes wrong:**
The naive approach to 3D-in-SVG sorts polygons by their Z-centroid and emits them back-to-front into the SVG document. This works for convex surfaces (a sphere, a simple bowl) but fails silently for non-convex meshes, saddle surfaces, and any camera angle where mesh panels cycle-overlap each other. The rendered output looks correct for some camera angles and wrong for others, with no error signal — just visually incorrect polygon occlusion.

For a GAM surface (which can be non-convex over data ranges), the cyclic overlap case is entirely possible: Panel A's centroid is behind Panel B, which is behind Panel C, which is behind Panel A. Centroid-sorting has no correct answer for this topology.

**Why it happens:**
Developers arrive from web/2D backgrounds where document order handles z-ordering trivially. The centroid-sort approach is the first thing that comes to mind and produces plausible-looking output during early testing when the camera is in a "safe" position. The failure only surfaces at specific camera angles that expose the non-convexity — which are often exactly the interesting angles for GAM surfaces.

**How to avoid:**
1. Commit to centroid-sort with explicit documented limitations — this is the correct choice for eidos given the SVG pipeline constraint. GAM surfaces over a regular grid are approximately convex for typical viewpoints, making centroid-sort acceptable.
2. Implement back-face culling as a first-class optimization: compute the dot product of each face normal with the camera direction vector, and skip faces where the dot product is positive (facing away). This reduces polygon count, improves performance, and eliminates most wrong-order artifacts on the back of the surface.
3. Document the known failure case: "Painter's algorithm is used. For highly non-convex surfaces or camera angles that intersect multiple overlapping faces, some z-ordering artifacts may appear. This is a fundamental limitation of SVG-based 3D rendering without a z-buffer."
4. If correctness is required for a specific case, BSP (binary space partitioning) is the next step — but this is scope for v1.2+, not v1.1.

**Warning signs:**
- "This looks correct from the front but wrong from the side" reports during visual testing.
- Any camera angle between 60 and 90 degrees elevation on a saddle or ridge surface.
- Polygons that appear to be in front of themselves ("self-intersection" visual artifacts).

**Phase to address:**
3D mesh rendering phase (first phase of v1.1). Back-face culling and centroid-sort must be the foundation before surface shading or animation is added.

---

### Pitfall 2: Gimbal Lock from Euler Angle Camera Representation

**What goes wrong:**
Camera orbit is represented as (azimuth, elevation, roll) Euler angles. When elevation approaches ±90° (looking straight down or up at the surface), the azimuth and roll axes align, and the camera loses a degree of freedom. The surface appears to "snap" or "stutter" as the camera passes through the pole. For animated camera orbits (a key v1.1 feature), this manifests as jitter or spinning artifacts during the pole-pass transition.

The insidious variant: using quaternions for storage but constructing them by composing three separate rotations (Euler-style). This reproduces gimbal lock even with quaternion internals, because the construction method — not the storage format — is the root cause.

**Why it happens:**
Euler angles are the intuitive representation (azimuth/elevation/roll maps to how people think about "camera position"), so developers reach for them first. The failure only appears when animating through pole regions, which may not be tested during initial development.

**How to avoid:**
1. Use a quaternion internally for camera orientation, but accept azimuth/elevation from users at the API boundary for ergonomics.
2. Convert user-supplied azimuth/elevation to a quaternion **once** at input time by constructing a rotation around a world-axis (not by composing Euler rotations): build a quaternion for the azimuth rotation around the world-Y axis, then build a separate quaternion for the elevation rotation around the resulting right-vector, and multiply them.
3. For animated camera orbits (keyframe A to keyframe B), interpolate using SLERP (Spherical Linear Interpolation) between the two quaternions — never by linearly interpolating the Euler angles. SLERP guarantees constant angular velocity and avoids pole-region discontinuities.
4. Add a quaternion normalization step after SLERP to prevent floating-point drift over many frames.

**Warning signs:**
- Camera appears to "spin" when elevation is near 90°.
- Orbit animations stutter or snap at the top/bottom of a vertical orbit.
- `azimuth` or `elevation` fields stored directly on the camera struct (rather than `orientation: Quaternion`).

**Phase to address:**
Camera math phase. Must be correct before camera rotation animation is built. The quaternion representation is a prerequisite for the SLERP interpolation in the animation phase.

---

### Pitfall 3: Coordinate Space Proliferation Without a Single Transform Chain

**What goes wrong:**
3D surface rendering introduces three distinct coordinate spaces on top of eidos's existing 2D pipeline:

1. **Data space**: the raw `(x, y, z)` values from the user's dataset
2. **3D world space**: normalized coordinates for the 3D scene, typically centered and scaled to a unit cube
3. **Camera/view space**: world space rotated so the camera is at the origin looking along -Z
4. **Screen space (2D SVG)**: the projected x/y pixel coordinates after perspective divide

Plus eidos already has:

5. **Axes space**: the 2D plot coordinate system with tick-adjusted bounds (used by `plot_bounds()`)
6. **SVG/pixel space**: 1:1 mapping as established by the v1.0 `build_svg_document` viewBox setup

With six spaces in play, developers distribute transform logic across multiple structs. Data-to-world normalization appears in the surface constructor, camera rotation happens in a math helper function, perspective projection happens in SVG generation, and axis label placement is computed separately. The transforms are never wrong individually — but when composed in the wrong order, or when a future change touches one transform without updating the chain, subtle visual bugs appear (labels slightly off, data points not aligning with the surface).

**Why it happens:**
Each piece of code feels like it's doing "its part" of the transform. The layering of 2D and 3D spaces is genuinely complex and there's no single obvious place to own the full chain.

**How to avoid:**
1. Create a single `Camera3D` struct that owns the entire transform chain as a pipeline of explicit steps: `data_to_world()`, `world_to_view()`, `view_to_clip()`, `clip_to_screen()`. Each step is a pure function, testable in isolation.
2. No 3D coordinate transform logic appears anywhere outside `Camera3D`. Surface mesh vertices, scatter points, and axis label anchor points all call into `Camera3D` for projection — never compute their own transforms.
3. Maintain strict naming discipline in code: variables named `p_data`, `p_world`, `p_view`, `p_screen` so the space of each coordinate is visible at the call site.
4. The existing 2D coordinate system (`axes.plot_bounds()`, SVG viewBox) is untouched. The 3D pipeline outputs to the same SVG/pixel space that 2D primitives already use, so 3D and 2D elements compose naturally.

**Warning signs:**
- Any function with "projection" or "transform" in its name that isn't on `Camera3D`.
- Scatter point positions not aligning with the corresponding surface location when rendered together.
- Axis grid lines (projected 3D grid) not aligning with axis tick label positions.
- Ad-hoc `y = height - y` flips appearing in surface rendering code (y-axis sign confusion).

**Phase to address:**
Camera math phase. Define `Camera3D` and its transform chain as the first concrete output. All subsequent features (surface, scatter, animation) are built on top of it.

---

### Pitfall 4: SVG Polygon Count Performance Wall at Video Framerates

**What goes wrong:**
A 3D surface mesh over a 20x20 grid generates 800 triangles (or 400 quads). At 30fps for a 5-second camera orbit animation, that is 150 frames × 800 polygons = 120,000 SVG polygon elements to generate, parse, and rasterize. Each frame requires: computing 400 vertex projections, sorting 800 polygons by centroid depth, constructing an SVG string with 800 `<polygon>` elements, parsing that string in resvg/usvg, and rasterizing with tiny-skia.

The SVG string generation and XML parsing overhead at this scale is significant. Initial testing at 10x10 grid resolution will feel fine; the performance wall appears when the user increases mesh resolution for a smooth-looking surface.

eidos's existing SVG pipeline already noted this risk (v1.0 PITFALLS.md Pitfall 4). For 2D primitives the problem was theoretical; for 3D mesh it is near-certain at any reasonable mesh resolution.

**Why it happens:**
The pipeline is designed correctly for 2D with O(10s) of primitives per frame. The 3D mesh adds O(100s-1000s) primitives in a single render call, which the pipeline was not dimensioned for.

**How to avoid:**
1. Establish a performance baseline early: render a 30x30 mesh at 30fps, measure wall-clock time per frame. This tells you whether the SVG pipeline is viable for the target mesh size.
2. Bound the maximum mesh resolution in the API: `SurfaceMesh::new(data, resolution: u32)` where `resolution` defaults to 20 and is documented with a performance table. A 20x20 grid (800 triangles) is the practical ceiling for the SVG pipeline.
3. Pre-compute expensive per-frame work: the perspective projection matrix and the sort key computation can be decomposed so only vertex transforms (not the full matrix) change per camera angle. Cache the mesh topology (which vertices form which triangles) as a fixed structure.
4. Use back-face culling aggressively. A surface viewed from above with a 45° elevation angle exposes approximately half its faces to the camera — culling the back half halves polygon count before sorting.
5. If benchmarks show the pipeline is too slow even at 20x20, the mitigation is to generate SVG `<path>` elements (combining all back-face-culled polygons into a single `<path d="...">`) rather than individual `<polygon>` elements. This reduces the SVG node count from O(N²) to O(1) at the cost of losing per-face color/style control. This is a known technique for SVG-based 3D wireframes.

**Warning signs:**
- Frame generation time exceeds 100ms at 20x20 grid (would mean 30fps is impossible without significant optimization).
- SVG string length exceeds 500KB per frame (parse time becomes dominant).
- Wall-clock render time for a 5-second animation exceeds 3 minutes.

**Phase to address:**
3D mesh rendering phase, immediately after the first working prototype. Benchmark before adding surface shading or animation — regressions are invisible without a baseline.

---

### Pitfall 5: Back-Face Culling Skipped Because "It Works in Testing"

**What goes wrong:**
The first prototype renders without back-face culling and looks correct for the default camera angle (e.g., 45° azimuth, 30° elevation, looking at the top of the surface). All 800 triangles are sorted and drawn, including the underside of the surface that is never visible. The render is twice as slow as it needs to be and the underside polygons are partially visible through the transparent portions of the mesh when the surface has gaps.

Back-face culling is then added as a "nice-to-have" optimization but deprioritized when other features need building. It stays skipped indefinitely.

**Why it happens:**
Back-face culling requires computing per-face normals (or using the cross product of projected edge vectors) before the sort step. It's a few extra lines but not immediately visible as a problem.

**How to avoid:**
Make back-face culling part of the initial polygon projection loop, not a post-hoc optimization. The test is cheap: `dot(face_normal_world, camera_direction_world) > 0.0` skips the face. Implement it at the same time as the centroid-depth sort — they use the same per-face normal computation.

**Warning signs:**
- Render time unexpectedly slow relative to visible polygon count.
- Slight polygon artifacts visible "through" the surface at steep viewing angles.
- Face normal computation only appears in one place (the shading function) rather than also being used for culling.

**Phase to address:**
3D mesh rendering phase, in the same implementation pass as polygon sorting.

---

### Pitfall 6: Surface Fitting Animation with Mismatched Vertex Topology

**What goes wrong:**
The surface fitting animation morphs from a flat plane to the fitted GAM surface. This requires interpolating vertex positions between the "flat" state (all z=0, or z=mean) and the "fitted" state (z=predicted_value). This only works correctly if the flat state and the fitted state have identical vertex count, identical grid topology, and vertex correspondence (vertex i in flat corresponds to vertex i in fitted).

If the flat plane is constructed separately from the fitted surface (e.g., `SurfaceMesh::flat()` and `SurfaceMesh::from_data()` take different resolution parameters), the vertex counts differ and interpolation is undefined. The animation either panics (if a bounds check exists) or silently renders garbage (if it does not).

**Why it happens:**
The flat plane and the fitted surface feel like different objects conceptually, and the user might specify different mesh resolutions for each. The vertex correspondence constraint is non-obvious from the API.

**How to avoid:**
1. Make vertex correspondence structurally enforced: `SurfaceMesh::fitting_animation(flat_z_fn, fitted_z_fn, grid_size)` constructs both the start and end mesh from the same grid topology. There is no separate "flat mesh" constructor — the flat is always derived from the same grid as the fitted.
2. The interpolation function takes `(start_mesh, end_mesh, t)` and asserts `start_mesh.vertex_count() == end_mesh.vertex_count()` with a clear error message: `"Cannot interpolate surface meshes with different vertex counts: {} vs {}. Both surfaces must be constructed from the same grid."`.
3. The `SurfaceMeshState` struct (used by the animation engine's Tween) stores only z-values as a `Vec<f64>`, with x/y grid coordinates fixed. Interpolation is then z-only, which eliminates the topology mismatch problem entirely.

**Warning signs:**
- Two separate constructors that produce `SurfaceMesh` with potentially different `resolution` parameters.
- Animation code that iterates both meshes in parallel without a length assertion.
- `Vec<Vertex3D>` being the interpolated type rather than `Vec<f64>` (z-only changes during fitting).

**Phase to address:**
Surface fitting animation phase. The z-only interpolation design should be established when defining the `SurfaceMeshState` type, before building the animation system on top of it.

---

### Pitfall 7: Axis Labels and Tick Marks Rendered at Wrong SVG Depth (Always On Top or Always Behind)

**What goes wrong:**
The 3D plot has axis lines, tick marks, and tick labels for three axes (X, Y, Z). These are not part of the mesh — they are separate SVG elements. In the SVG document, all elements are rendered in document order. If axis labels are added to the SVG before the mesh polygons, they render behind the mesh. If added after, they always render on top of the mesh — including when the axis is geometrically behind the surface.

For a typical camera angle looking at a surface from above-and-side, the far axis edges should be partially occluded by the surface. If all axis elements are always on top, the visualization looks wrong: axis lines float in front of a surface that should be in front of them.

**Why it happens:**
SVG has no z-buffer. The standard SVG pipeline for 2D elements (where document order is render order) breaks for any 3D element that has a varying depth relationship with the mesh.

**How to avoid:**
1. For v1.1, use the pragmatic approach: axis grid lines and tick marks are sorted and inserted into the polygon emission order along with mesh faces. Each axis edge segment becomes a "polygon" in the back-to-front sort with its own centroid depth. This correctly occludes far axis edges behind near mesh faces.
2. Tick labels (text) are always rendered last (on top), because text partially occluded by a mesh is unreadable and confusing. Accept this simplification and document it.
3. The axis structure for 3D is separate from the existing `Axes` type — it generates projected 3D line segments, not 2D SVG paths. The existing 2D `Axes` type is unchanged.

**Warning signs:**
- "The axis labels are behind the surface" or "The axis lines are in front of the surface" in visual testing.
- Axis SVG elements added unconditionally before or after all mesh elements in the document construction function.
- Trying to use the existing 2D `Axes` struct to draw 3D axis lines by passing projected coordinates (the tick layout and labeling math is fundamentally 2D).

**Phase to address:**
3D mesh rendering phase, when implementing the axis frame. The sort order must account for axis elements from the start.

---

### Pitfall 8: 3D API Feels Grafted On (Mismatched Mental Model With 2D API)

**What goes wrong:**
The existing eidos API is:
```rust
scene.render(|s, t| {
    s.add_axes(&axes);
    s.add(curve);
}, "output.mp4")?;
```

A 3D surface API that feels natural might be:
```rust
scene.render(|s, t| {
    s.add_surface(&surface);
}, "output.mp4")?;
```

But the 3D camera state does not fit into `add()` or `add_axes()`. A camera must be configured, the surface projection depends on the camera, and animated camera rotation means the camera changes per-frame. If the camera is a field on the `SceneBuilder`, the 2D/3D split becomes confusing. If it's passed as an argument to `add_surface()`, the API is inconsistent with `add()`. If it's a separate `add_camera()` call, users may forget to add it.

The deeper problem: the 2D mental model is "objects in a flat scene." The 3D mental model is "camera looking at objects in a world." These are fundamentally different, and a single `SceneBuilder` API cannot serve both naturally.

**Why it happens:**
The temptation to reuse `SceneBuilder` for 3D content is strong because it avoids branching the API. But the 3D case has state (camera) that does not exist in 2D, and forcing it into the 2D builder creates awkwardness that compounds as 3D features grow.

**How to avoid:**
1. Introduce a separate `Scene3D` or `SurfacePlot` builder that wraps its own camera and renders to a `Primitive::SurfaceMesh` (a new enum variant). The user's entry point for 3D is `SurfacePlot::new()`, not `SceneBuilder`.
2. The `SurfacePlot` renders to a list of 2D `Primitive` values (sorted polygons, axis segments, text labels) that are handed back to `SceneBuilder` via `s.add_surface_plot(&plot, t)`. The 3D-to-2D projection is encapsulated entirely inside `SurfacePlot`.
3. This mirrors the existing pattern: `Axes` is a 2D construct that `to_primitives()` into a list of `Primitive` values. `SurfacePlot` is the 3D equivalent — a high-level object that projects itself to 2D primitives.
4. The camera animation is expressed as a `Tween<CameraState>` stored on the `SurfacePlot`, consistent with how the animation engine works elsewhere in eidos.

**Warning signs:**
- A `camera: Option<Camera3D>` field on `SceneBuilder`.
- `SceneBuilder::add_camera()` as a method.
- 3D API examples that require more setup steps than the equivalent 2D API examples.
- Needing to pass `t` into `add_surface()` but not into `add_axes()`.

**Phase to address:**
API design phase. The `SurfacePlot` vs `SceneBuilder` boundary must be settled before any surface rendering code is written, because it determines the ownership and lifetime of the camera.

---

### Pitfall 9: Scatter Point Z-Depth Not Participating in Polygon Sort

**What goes wrong:**
Data scatter points (`(x, y, z)` raw data projected to screen positions) are added as SVG circle elements. They are placed in the SVG document after all mesh polygons. This means scatter points always render on top of the surface, even when a data point is geometrically behind the surface from the current camera angle.

For a GAM visualization, a scatter point at `(x=0.5, y=0.5, z=low)` on a region where the fitted surface is high should be occluded by the surface. If the point always renders on top, the visualization incorrectly implies it is in front.

**Why it happens:**
Scatter points are "obviously circles" that belong to the SVG `<circle>` primitive type. The mesh polygons are sorted, but the scatter points are added as a separate post-hoc step without participating in the same sort.

**How to avoid:**
1. Scatter points participate in the centroid-depth sort alongside mesh face polygons. Each scatter point is an entry in the sort list with its projected Z depth as the sort key.
2. Scatter points render as small SVG `<circle>` elements, emitted at their correct position in the sorted draw order — after mesh faces that are behind them, before mesh faces in front of them.
3. The draw order list is heterogeneous: `Vec<DepthSortedElement>` where `DepthSortedElement` is an enum of `{ MeshFace(polygon), ScatterPoint(circle) }`. The sort is on the `depth` field of this enum, then dispatch to the SVG emission for the appropriate type.

**Warning signs:**
- Scatter points added to the SVG document in a separate loop after the polygon-emission loop.
- Scatter points always appearing in front of the mesh regardless of camera angle.
- No `depth` field or sort key on the scatter point data structure.

**Phase to address:**
Data scatter phase (the scatter feature phase). The depth-sorted heterogeneous draw list design should be established when scatter is first introduced.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Centroid-sort for all geometry including scatter points | Simple unified sort | Wrong Z-order for points at non-centroid positions (a point's centroid IS its position, so this is actually correct) | Always acceptable for point geometry; only problematic for large polygons |
| Hard-coding mesh resolution (e.g., 20x20) | No performance tuning needed | Users with fine-grain surfaces cannot get smooth rendering | Acceptable for v1.1 with documented limit; expose `resolution` parameter in v1.2 |
| Euler angles in user-facing API even though quaternions used internally | Intuitive for users | Documentation must explain that camera API uses degrees not radians | Always acceptable at the boundary layer |
| Always rendering tick labels on top (not depth-sorted) | Avoids complex text depth sorting | Slight visual inaccuracy for extreme camera angles | Acceptable; the alternative (occluded text) is worse UX |
| Skipping BSP tree for z-ordering | Saves significant complexity | Incorrect rendering for highly non-convex surfaces | Acceptable for v1.1 GAM surfaces which are approximately convex in practice |
| Reusing `Primitive` enum for mesh face polygons (as `Primitive::Polygon`) | Fits existing dispatch pattern | Primitive enum grows without bound as 3D features add new types | Never; prefer `SurfacePlot::to_primitives()` returning `Vec<Primitive>` |

---

## Integration Gotchas

Common mistakes when connecting 3D surface rendering to the existing eidos pipeline.

| Integration Point | Common Mistake | Correct Approach |
|-------------------|----------------|------------------|
| `build_svg_document` + 3D mesh | Passing 800 `Primitive::Polygon` entries to `build_svg_document` in random order | Sort polygons by depth before calling `build_svg_document`; the SVG function emits in order |
| `Tween<T>` animation engine + `CameraState` | Making `CameraState` hold Euler angles and interpolating linearly | Store quaternion orientation in `CameraState`, implement `Lerp for CameraState` using SLERP |
| `SceneBuilder::add()` + `SurfacePlot` | Calling `s.add(surface_plot)` directly (makes SceneBuilder 3D-aware) | `SurfacePlot::to_primitives(t)` returns `Vec<Primitive>`, which are individually `add()`ed |
| Existing 2D `Axes` + 3D plot | Reusing `Axes` to draw X/Y axis labels on a 3D scene | 3D axis is a separate type; does not inherit from `Axes`; projects its own tick positions |
| Per-frame mesh vertex projection | Recomputing full MVP matrix every frame inside the closure | Compute projection matrix once outside the frame closure; only update if camera changes |
| Color by value (surface shading) | Mapping Z-value to color using the full `Color` HSL conversion per polygon | Precompute the color map as a lookup table over the Z range before rendering |

---

## Performance Traps

Patterns that work at small mesh resolution but fail at realistic resolution.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Allocating `Vec<f64>` for z-values per frame in morph animation | Excessive heap allocation per frame at 30fps | Pre-allocate morph z-buffer once; reuse across frames with in-place lerp | At 30fps, 30x30 grid: 27,000 allocs/sec |
| Sorting full polygon list including back-facing polygons | Sort takes 2x longer than necessary | Cull back-faces before sort; only sort visible polygons | Visible from ~400 polygon grids |
| String-building SVG for each polygon separately with `format!()` | SVG generation becomes string allocation bottleneck | Use a `String` pre-allocated to estimated capacity; write polygon coords directly | At 800+ polygons per frame |
| Computing face normals from scratch each frame | Redundant work for static mesh topology | Precompute face normals in mesh struct; only reproject to world space per camera | At any mesh resolution with rotating camera |
| Using f32 for projection math | Vertex positions jitter at extreme zoom or large coordinate values | Use f64 throughout the projection pipeline; cast to f32 only at SVG coordinate output | When data range exceeds ±1000 |

---

## UX Pitfalls

Common user experience mistakes specific to 3D surface API design.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Camera specified as distance + azimuth + elevation with no defaults | Users must always specify all 3 values | Provide a `Camera3D::default()` that gives a useful "looking at the surface from above-left" angle |
| Surface color range not matching data range by default | Surface looks uniformly colored or clipped at extremes | Auto-compute color map range from the z min/max of the provided data, same as `Axes` auto-range |
| Morph animation that starts from z=0 (flat at the plot origin) | The flat baseline may be outside the visible data range if z values are all positive | Default flat baseline to z=mean of the data, not z=0 |
| No indication of axis orientation for first-time users | Users cannot tell which axis is X, Y, or Z from the output | Emit axis labels ("X", "Y", "Z" by default) and directional arrowheads on the axis lines |
| Camera orbit animation that jumps discontinuously at loop boundary | Video looks broken when played as a loop | Ensure orbit animation start and end quaternions are the same (full 360° orbit); verify with a short loop test |

---

## "Looks Done But Isn't" Checklist

Things that appear complete during development but are missing critical pieces.

- [ ] **Polygon depth sort:** Visually correct from the default camera angle does not mean correct from all angles. Test from at least 8 camera positions (4 azimuths × 2 elevations) before considering the sort implementation done.
- [ ] **Camera orbit animation:** A rotating camera that "looks smooth" may still have gimbal lock at 90° elevation. Test with a polar orbit (elevation sweeps from -90° to +90°) to confirm no stuttering.
- [ ] **Scatter point depth:** Scatter points "appearing" on the surface does not mean they are correctly depth-sorted. Verify with a data point that should be behind the surface from the test camera angle.
- [ ] **Surface morph animation:** The flat-to-fitted morph "looking like it morphs" does not confirm vertex correspondence. Verify by checking that vertex count of start and end states are identical.
- [ ] **Axis labels in 3D:** Axis labels "showing up" on screen does not mean they are at the correct 3D position. Verify the projected tick positions against manually calculated world-to-screen coordinates for known data values.
- [ ] **Back-face culling:** The surface "looking correct" does not confirm culling is active. Add a debug render mode that colors back-facing polygons red to verify the cull boundary.
- [ ] **Performance at target resolution:** A 10x10 grid rendering fast does not predict 30x30 performance. Benchmark at the maximum documented resolution before the phase is complete.

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Gimbal lock discovered in camera animation | MEDIUM | Replace Euler angle interpolation with quaternion SLERP; the camera storage type changes but user API can stay the same |
| Z-order artifacts discovered at specific camera angles | LOW | Document the limitation; add back-face culling if not already present; consider BSP tree for v1.2 |
| SVG performance wall at target mesh resolution | HIGH | Refactor polygon emission to use `<path>` combined elements for wireframe mode; surface fill becomes a single `<path>` per face row; this changes SVG generation significantly |
| Vertex count mismatch in morph animation | MEDIUM | Enforce grid-topology-derived meshes for both start and end states; add assertion with clear error message |
| API feels grafted on (found during user testing) | HIGH | Introduce `SurfacePlot` wrapper type; migrate camera out of `SceneBuilder`; this changes public API requiring a minor version bump |
| Scatter points always on top | LOW | Add scatter points to the depth-sorted draw list; small refactor to make the draw list heterogeneous |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Painter's algorithm failure (#1) | 3D mesh rendering (phase 1) | Test from 8 camera angles including non-convex mesh regions |
| Gimbal lock (#2) | Camera math (phase 1) | Polar orbit test: sweep elevation from -90° to +90° with continuous animation |
| Coordinate space proliferation (#3) | Camera math (phase 1) | Unit test each `Camera3D` transform step; verify scatter and mesh agree on projected position |
| SVG performance wall (#4) | 3D mesh rendering (phase 1), benchmark | Render 30x30 mesh at 30fps; measure per-frame time |
| Back-face culling skipped (#5) | 3D mesh rendering (phase 1) | Debug render with back-faces colored red; confirm ~50% culled at 45° elevation |
| Surface morph vertex mismatch (#6) | Surface fitting animation phase | Assertion test: constructing morph from two different grid sizes triggers clear error |
| Axis labels wrong depth (#7) | 3D mesh rendering (phase 1) | Visual test: axis behind surface is occluded; axis in front is visible |
| 3D API feels grafted on (#8) | API design (before any implementation) | User-test the proposed API with a representative example before writing rendering code |
| Scatter points wrong depth (#9) | Data scatter phase | Visual test: point at `z < surface_z` is behind surface from above camera angle |

---

## Sources

- [Painter's Algorithm -- Wikipedia](https://en.wikipedia.org/wiki/Painter%27s_algorithm): Cyclic overlap and piercing polygon failure cases
- [3D Wireframes in SVG -- prideout.net](https://prideout.net/blog/svg_wireframes/): Centroid-sort implementation, back-face culling via winding, SVG polygon ordering for 3D
- [Gimbal Lock -- Wikipedia](https://en.wikipedia.org/wiki/Gimbal_lock): Root cause analysis
- [Quaternion Rotation -- Medium (Ralf Becker)](https://medium.com/@ratwolf/quaternion-3d-rotation-32a3de61a373): Why quaternion storage alone does not fix gimbal lock
- [Gimbal Lock with Quaternions -- Unity Discussions](https://forum.unity.com/threads/is-quaternion-really-do-not-suffer-from-gimbal-lock.555829/): The construction-vs-storage distinction
- [LearnOpenGL -- Coordinate Systems](https://learnopengl.com/Getting-started/Coordinate-Systems): The five-space transform chain (object, world, view, clip, screen)
- [Multiple Coordinate Spaces -- 3D Math Primer](https://gamemath.com/book/multiplespaces.html): Active vs passive transform confusion
- [Spherical Linear Interpolation -- Wikipedia](https://en.wikipedia.org/wiki/Slerp): SLERP constant angular velocity guarantee
- [SVG vs Canvas vs WebGL Performance -- SVGGenie](https://www.svggenie.com/blog/svg-vs-canvas-vs-webgl-performance-2025): SVG DOM overhead limits, >5000 nodes causes lag
- [Mesh Morphing Vertex Correspondence -- GameDev.net](https://www.gamedev.net/forums/topic/679147-shape-interpolation/): Identical vertex count requirement for morph animation
- [BSP Tree Polygon Sorting -- GeeksforGeeks](https://www.geeksforgeeks.org/dsa/painters-algorithm-in-computer-graphics/): BSP as the correct solution to painter's algorithm failures
- [Back-Face Culling -- Wikipedia](https://en.wikipedia.org/wiki/Back-face_culling): Winding order and dot product test
- [LearnOpenGL -- Face Culling](https://learnopengl.com/Advanced-OpenGL/Face-culling): Winding order consistency requirement
- [The Perspective and Orthographic Projection Matrix -- Scratchapixel](https://www.scratchapixel.com/lessons/3d-basic-rendering/perspective-and-orthographic-projection-matrix/building-basic-perspective-projection-matrix.html): Projection matrix construction and near/far clipping

---
*Pitfalls research for: Adding 3D surface rendering to eidos v1.1 (existing 2D SVG Rust animation library)*
*Researched: 2026-02-25*
