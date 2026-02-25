# Feature Research

**Domain:** 3D surface visualization for data visualization / ML presentation tools (v1.1 milestone — adding to existing Rust animation library)
**Researched:** 2026-02-25
**Confidence:** MEDIUM-HIGH

> **Scope note:** v1.0 already ships 2D axes, DataCurve, ConfidenceBand, SplineFit, and all animation primitives.
> This document covers ONLY features needed for the v1.1 3D surface milestone. The existing 2D features are not repeated here.

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist in a 3D surface visualization tool. Missing any of these and the 3D plot feels incomplete or broken.

#### Mesh Rendering

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Wireframe mesh rendering | Every 3D surface tool ships wireframe (matplotlib `plot_wireframe`, plotly, MATLAB `surf`). It's the universal "I can see the shape" representation. | MEDIUM | Project 3D grid vertices to 2D via perspective transform. Draw edges as line segments. Row/column stride controls density. Painter's algorithm (z-sort faces back-to-front) handles occlusion in SVG pipeline without a depth buffer. |
| Shaded solid surface | Users expect to see filled polygons with depth cues, not just wireframe edges. matplotlib `plot_surface`, plotly surface traces. | MEDIUM | Fill each quad/triangle face with a color. Z-height colormap (face color = function of z-value) is the standard. Flat shading per face is sufficient; Gouraud interpolation is a differentiator. |
| Z-height colormap coloring | Nearly universal default: color encodes the z-value using a gradient (e.g., cool-to-warm). Without this, surfaces look flat and hard to read. | LOW | Map each face's mean z-value to a color using a configurable palette. Only requires a linear interpolation between two or more color stops. |
| 3D cartesian axes (x, y, z) | Users expect labeled axes with ticks on all three dimensions. Without them, the viewer has no spatial reference. | MEDIUM | Extend existing `Axes` concept to 3D. Three axis lines projected to 2D. Tick labels at the axis extremes. Complexity: deciding which axis edges to draw (back-facing vs front-facing edges of the bounding box). |
| Configurable viewpoint (azimuth + elevation) | Both matplotlib and plotly expose `view_init(elev, azim)` / `scene_camera.eye`. Users need to control the "angle they're looking from" before rendering. | LOW | Store (azimuth, elevation, distance) as camera parameters. Translate to a view matrix at render time. This is just a state value on the surface object — no animation yet. |
| Axis labels and tick marks in 3D | Even minimal 3D plots in matplotlib/plotly show axis labels. Without them, the viewer cannot read off values. | MEDIUM | Project axis positions to 2D. Reuse existing text-label machinery from v1.0. Correct label placement is the hard part: labels should appear on the visible (outward-facing) edges. |

#### Data Point Scatter in 3D

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Scatter points at (x, y, z) | Any ML surface plot overlays the raw training data as scatter points. matplotlib `scatter`, plotly `scatter3d`. Standard in PDP/GAM presentations to show where the surface was fit to. | MEDIUM | Project each (x, y, z) point to 2D using the same perspective transform as the mesh. Depth-sort against mesh faces. Core challenge: partial occlusion by the surface (see pitfalls). |
| Depth-based opacity for scatter | matplotlib's `depthshade=True` is default — scatter markers fade based on distance from viewer to give depth cues. Without it, distant points look equally prominent as near ones. | LOW | Scale alpha of each point marker by its projected z-depth. Simple linear scale from 0.3 to 1.0 opacity based on depth percentile within the point set. |
| Configurable marker size and color | Users assign color to distinguish point groups or encode a 4th variable. Standard in plotly scatter3d (marker.size, marker.color). | LOW | Marker size (radius) and fill color as struct fields with sensible defaults. Color can be uniform or per-point from a Vec<Color>. |

#### Camera and Viewpoint Control

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Orbit camera animation (rotation around z-axis) | The canonical "show me the surface from all angles" animation. Widely used in plotly, Blender, rayshader. For video output, this is the primary way to convey 3D structure to a passive viewer. | MEDIUM | Animate azimuth angle from start to end over a frame range using existing Tween infrastructure. The camera circles the surface while elevation stays fixed. Each frame: reproject with updated azimuth. |
| Elevation swing animation | Complement to rotation — animate the viewing angle up/down to reveal overhanging or undercut regions. | LOW | Same as orbit but animating elevation angle. Can compose with azimuth animation. Uses existing Tween infrastructure. |
| Smooth camera easing | Constant-speed rotation looks mechanical. Ease-in/ease-out on orbit angle makes the animation cinematic. matplotlib rotation animations are often constant-speed and look robotic. | LOW | Apply existing Easing variants to the azimuth/elevation Tween. No new infrastructure — this comes for free once orbit animation uses Tween. |

#### Surface Fitting Animation

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Flat-to-fitted surface morph | Extending the existing SplineFit animation to 3D: mesh begins flat (all z = 0) and deforms to the final fitted surface. This is the v1.1 differentiator. | HIGH | Interpolate z-values of each mesh vertex from 0.0 to final z at each frame. Uses existing Tween/frame-time morphing pattern from SplineFit v1.0. Per-vertex z interpolation, not per-face. |
| Frame-time control (morph duration/start) | SplineFit v1.0 uses a `frame_time` parameter. Users expect the same control for 3D morphing: when does the surface start fitting and when does it complete. | LOW | Mirror the SplineFit API pattern. `start_frame`, `end_frame`, `easing` parameters on the surface object. Reuses existing infrastructure. |

---

### Differentiators (Competitive Advantage)

Features that set eidos apart in this domain. Not universally expected, but high value.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Synchronized orbit + surface morph | Camera orbiting *while* the surface fits from flat to final shape. No 3D visualization tool does this in a single declarative spec — it requires carefully sequenced keyframes in plotly/matplotlib. eidos's composable animation model makes this natural. | MEDIUM | Compose orbit Tween with surface morph Tween using the existing parallel/sequential composition model. Both animate simultaneously over their respective frame ranges. |
| Wireframe + filled hybrid rendering | Show a shaded surface *with* a wireframe grid overlay at reduced opacity. matplotlib does this by layering two separate plot calls; eidos can expose it as a single `surface_style: WireframeOverlay` option. | LOW | Render filled faces first (back-to-front), then render grid lines on top. Toggle via enum: `Wireframe`, `Filled`, `FilledWithGrid`. |
| Data points appear via animation | Scatter points fade or drop in after the surface settles — creating a narrative "here's the model, here's the data it came from." Tools like plotly display everything at once; eidos can sequence these. | LOW | Scatter points as first-class animated objects. FadeIn or DropIn (fall from above the surface) animation using existing Tween infrastructure. |
| Contour projection on base plane | Project iso-z contour lines onto the z=zmin plane below the surface. Plotly supports this but requires manual configuration; rayshader's shadows achieve a similar depth effect. Adds strong depth cue for viewers. | MEDIUM | Compute z-contours from the mesh, project them onto a floor plane, render as faint lines. Helps still-frame viewers understand the surface shape before rotation. |
| Consistent visual identity with 2D plots | eidos 2D plots (dark background, smooth color palette, clean vector aesthetics) should carry through to 3D surfaces. Most 3D tools default to a jarring style change. | LOW | Reuse existing Color and styling infrastructure. Default colormap, background color, and line weights should match the 2D visual language. |

---

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem desirable but create problems in this context.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Interactive rotation (mouse drag) | Users of plotly/matplotlib expect to spin the plot in their viewer. Feels essential for exploration. | eidos is video-only — no event loop, no runtime interactivity. Adding interaction would require a completely different architecture (window/event loop, GPU rendering, runtime state). Fundamentally out of scope. | Render multiple videos from different viewpoints (azimuth: 0°, 45°, 90°, 135°). Or use orbit animation to show all angles in one video. |
| Depth buffer / GPU rasterization | Proper per-pixel depth testing would eliminate all z-sorting artifacts. GPU rendering (wgpu, etc.) is tempting once you have 3D. | Completely breaks the SVG pipeline that is eidos's foundation. Requires GPU dependencies, OpenGL/Vulkan context, and a totally different rendering architecture. Adds enormous complexity. | Painter's algorithm (back-to-front z-sort of faces) is sufficient for convex or near-convex surfaces like GAM output. Flag known limitations honestly. |
| Full lighting model (point lights, shadows, ambient occlusion) | Rayshader's selling point is photorealistic lighting. Users see it and want it. | Full lighting requires normal vector computation, light-transport math, and is GPU-hostile in an SVG pipeline. Rayshader itself renders to PNG via ray tracing — not SVG. | Z-height colormap plus optional directional shading (lightsource: single directional light applied as brightness multiplier to face color) is sufficient for depth perception. matplotlib's `lightsource` parameter follows this pattern. |
| Arbitrary triangle mesh (non-grid topology) | `plot_trisurf` in matplotlib handles unstructured meshes. General meshes enable arbitrary geometry. | GAM surfaces are defined on regular (x,y) grids — no triangulation needed. Supporting arbitrary triangle meshes requires robust topology handling (adjacency, winding order, degenerate triangles) that is out of scope. | Regular grid mesh (NxM quads) covers all GAM/PDP surface use cases. Unstructured mesh can be a v2+ feature. |
| Logarithmic 3D axes | Common request from scientific users. | Axis label math becomes non-trivial. Tick placement requires log-space Heckbert algorithm extension. Low frequency of need for GAM/ML surfaces. | Implement log scaling as a data-preprocessing step (transform the z-values before passing to eidos). Document the pattern. |
| Animation of the colormap itself | Transitioning from one colormap to another (e.g., grayscale → cool-to-warm) as the surface fits. Visually striking. | Colormap animation requires per-frame recoloring of all faces and provides marginal interpretive value. Adds complexity without serving the core use case. | Animate z-values (the morph). Use a fixed colormap. The color changes naturally follow the z animation. |
| 4D data (color encodes 4th variable, animated over time) | Full generality of plotly's `frames` for animated time series. | Requires a fundamentally different data model (not just z = f(x,y) but z = f(x,y,t) with color = g(x,y,t)). Scope explosion. | Two-variable GAM surfaces are the target. Time-series animation can be composed using existing keyframe machinery for a specific use case when needed. |

---

## Feature Dependencies

```
Perspective Projection Transform
    └──required by──> Wireframe Mesh Rendering
    └──required by──> Shaded Surface Rendering
    └──required by──> Scatter Point Projection
    └──required by──> 3D Axis Lines

Wireframe Mesh Rendering
    └──required by──> Flat-to-Fitted Surface Morph (animates the wireframe z-values)

Shaded Surface Rendering
    └──required by──> Wireframe+Filled Hybrid (renders filled faces before grid lines)
    └──enhanced by──> Contour Projection (adds floor contours to same scene)

Z-Height Colormap
    └──required by──> Shaded Surface Rendering (each face needs a color)

Painter's Algorithm (z-sort)
    └──required by──> Correct occlusion of: mesh faces, scatter points, grid lines

Configurable Viewpoint (azimuth, elevation)
    └──required by──> Orbit Camera Animation (animates the azimuth parameter)
    └──required by──> Elevation Swing Animation (animates the elevation parameter)

Existing Tween + Easing (v1.0)
    └──re-used by──> Orbit Camera Animation (tween azimuth angle)
    └──re-used by──> Surface Morph Animation (tween per-vertex z-values)
    └──re-used by──> Scatter FadeIn Animation

Existing SplineFit frame_time pattern (v1.0)
    └──extends to──> Surface Morph Animation API design

Existing Text Label rendering (v1.0)
    └──re-used by──> 3D Axis Labels and Tick Marks

3D Cartesian Axes
    └──required by──> Scatter Point Scatter (tick marks give scale reference)
    └──enhanced by──> Axis Labels
```

### Dependency Notes

- **Perspective projection is the foundation:** All 3D features depend on a correct 3D-to-2D projection transform. This must be the first thing implemented. It is a pure math function (view matrix * perspective matrix * vertex -> screen xy).
- **Painter's algorithm is sufficient but imperfect:** Z-sorting faces back-to-front handles occlusion for convex surfaces. Matplotlib's `zorder` bugs (GitHub issues #14148, #23392, #5830) arise from this approach's fundamental limitations with non-convex geometry and scatter point interactions. eidos should document this limitation explicitly. For GAM surfaces (which are never self-intersecting), painter's algorithm produces correct results.
- **Surface morph reuses v1.0 SplineFit pattern:** The flat-to-fitted morph is conceptually identical to SplineFit but applied per-vertex in 3D. The `frame_time` → `t: f64 ∈ [0,1]` → interpolate pattern carries over directly.
- **Camera animation reuses v1.0 Tween:** Azimuth and elevation are just f64 values. The existing Tween + Easing infrastructure animates them directly. No new animation machinery is needed.
- **Scatter occlusion is the hard problem:** Correctly sorting scatter points relative to mesh faces is the most error-prone part. See PITFALLS.md. The recommended initial approach is to render all scatter above the surface, or all below, and document the limitation.

---

## MVP Definition

### Launch With (v1.1)

Minimum viable 3D surface that serves the GAM/ML presentation use case.

- [ ] **Perspective projection transform** — math foundation; everything else depends on it
- [ ] **Wireframe mesh rendering** — configurable (azimuth, elevation, distance), z-sorted, with grid density control
- [ ] **Z-height colormap coloring on filled surface** — shaded faces using z-value → color gradient
- [ ] **3D cartesian axes** — three axis lines with tick marks and labels, projected correctly
- [ ] **Flat-to-fitted surface morph animation** — z-values lerp from 0 to final shape over frame range using existing Tween infrastructure
- [ ] **Orbit camera animation** — azimuth sweeps while surface is displayed; uses existing Tween + Easing
- [ ] **Scatter points in 3D** — (x, y, z) points projected and rendered with depth-based opacity

### Add After Validation (v1.1.x)

Features to add once the core MVP is working and validated.

- [ ] **Wireframe + filled hybrid mode** — single option to overlay grid on shaded surface; low complexity, high visual value
- [ ] **Elevation swing animation** — complements orbit; easy addition once orbit animation works
- [ ] **Scatter FadeIn animation** — scatter points appear via fade after surface settles; reuses existing animation

### Future Consideration (v2+)

Features to defer until MVP is solid and a real use case demands them.

- [ ] **Contour projection on base plane** — high visual value but non-trivial; defer until base MVP is solid
- [ ] **Directional shading (lightsource)** — depth cue via brightness multiplier; useful but not essential
- [ ] **Unstructured triangle mesh** — needed only if data isn't on a regular grid; not required for GAM surfaces
- [ ] **Multiple surfaces in one scene** — overlay two model surfaces for comparison; requires careful z-sorting
- [ ] **Orthographic projection mode** — isometric/orthographic view as alternative to perspective; limited demand

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Perspective projection transform | HIGH | MEDIUM | P1 |
| Wireframe mesh rendering | HIGH | MEDIUM | P1 |
| Shaded surface + z-colormap | HIGH | MEDIUM | P1 |
| 3D cartesian axes | HIGH | MEDIUM | P1 |
| Flat-to-fitted surface morph | HIGH | HIGH | P1 |
| Orbit camera animation | HIGH | MEDIUM | P1 |
| Scatter points (x,y,z) | HIGH | MEDIUM | P1 |
| Depth-based scatter opacity | MEDIUM | LOW | P1 |
| Wireframe + filled hybrid | MEDIUM | LOW | P2 |
| Elevation swing animation | MEDIUM | LOW | P2 |
| Scatter FadeIn animation | MEDIUM | LOW | P2 |
| Contour projection on floor | MEDIUM | MEDIUM | P3 |
| Directional shading | LOW | MEDIUM | P3 |
| Unstructured triangle mesh | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for v1.1 launch
- P2: Should have, add when core is working
- P3: Nice to have, future consideration

---

## Competitor Feature Analysis

| Feature | matplotlib (mpl_toolkits.mplot3d) | plotly (3D surface) | rayshader (R) | eidos v1.1 approach |
|---------|-----------------------------------|---------------------|---------------|----------------------|
| Surface rendering | `plot_surface` with colormap, lightsource | Interactive surface trace, colorscale, opacity | Raytraced shading, photorealistic | Painter's algorithm on SVG, z-colormap, no GPU |
| Wireframe | `plot_wireframe`, rcount/ccount for density | Wireframe toggle on surface trace | Wireframe via separate layer | Grid density parameter, overlay mode |
| Camera control | `view_init(elev, azim)`, interactive in Jupyter | `scene_camera` with eye/center/up vectors | `render_camera` with phi/theta/zoom/fov | Azimuth + elevation as f64 params, animated via Tween |
| Camera animation | `FuncAnimation` with manual `view_init` update per frame | No built-in animation; use `frames` with layout updates | `render_movie_frames` scripted camera path | Declarative: orbit Tween over frame range |
| Scatter on surface | `scatter3D` separate call, persistent z-ordering bugs | `scatter3d` trace overlaid on surface trace | Overlaid as separate layer | Scatter as first-class 3D object, painter's sort |
| Surface morph animation | `FuncAnimation` + manual z-array update per frame | `frames` list of layout updates | Not available | Flat-to-fitted morph as first-class: frame_time API |
| Occlusion correctness | Known bugs: markers vs surface, global threshold issue (GitHub #23392, #5830) | WebGL depth buffer: pixel-correct | Ray tracing: pixel-correct | Painter's algorithm: correct for convex/non-intersecting; documented limitation for complex cases |
| Output format | Static PNG/SVG or embedded interactive HTML | Interactive HTML/JSON or static PNG | Static PNG (raytraced), MP4 via scripting | MP4 video (programmatic animation, no interaction) |
| Fitting animation | Not built-in; manual per-frame scripting | Not built-in; `frames` with data updates | Not available | First-class: matches SplineFit API design from v1.0 |

---

## Sources

- [matplotlib mplot3d stable docs](https://matplotlib.org/stable/users/explain/toolkits/mplot3d.html) — Feature inventory of mplot3d toolkit (MEDIUM confidence, official docs)
- [Plotly 3D surface plots (Python)](https://plotly.com/python/3d-surface-plots/) — Contour projection, colorscale, opacity, camera config (MEDIUM confidence, official docs)
- [Plotly 3D camera controls](https://plotly.com/python/3d-camera-controls/) — eye/center/up vectors, default camera params (HIGH confidence, official docs)
- [matplotlib GitHub issue #14148](https://github.com/matplotlib/matplotlib/issues/14148) — zorder ignored in mplot3d (HIGH confidence, primary source)
- [matplotlib GitHub issue #23392](https://github.com/matplotlib/matplotlib/issues/23392) — Axes3D distance from camera bug (HIGH confidence, primary source)
- [matplotlib GitHub issue #5830](https://github.com/matplotlib/matplotlib/issues/5830) — Incorrect scatter3D marker ordering (HIGH confidence, primary source)
- [rayshader (tylermorganwall)](https://www.rayshader.com/) — ggplot2 3D lifting, scripted camera animation, cinematic depth of field (MEDIUM confidence, official docs)
- [Painter's Algorithm overview](https://every-algorithm.github.io/2024/10/15/painters_algorithm.html) — Back-to-front sorting, limitations with intersecting polygons (MEDIUM confidence, secondary)
- [Python Data Science Handbook — 3D Plotting](https://jakevdp.github.io/PythonDataScienceHandbook/04.12-three-dimensional-plotting.html) — Surface + scatter combination patterns (MEDIUM confidence, secondary)
- [3D surface plot animations in Python](https://likegeeks.com/3d-surface-plot-animations-python/) — Surface morphing via FuncAnimation pattern (MEDIUM confidence, secondary)
- [Effective Multi-Dimensional 3D Scatterplots](https://arxiv.org/html/2406.06146v2) — Depth cues, fog, depth-based color mapping for scatter (MEDIUM confidence, academic)

---
*Feature research for: 3D surface visualization (eidos v1.1 milestone)*
*Researched: 2026-02-25*
