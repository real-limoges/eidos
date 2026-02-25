# Technology Stack

**Project:** eidos -- Manim-inspired Rust animation/visualization library
**Researched:** 2026-02-24 (v1.0) / 2026-02-25 (v1.1 addendum)

## Recommended Stack

### SVG Generation

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `svg` | 0.18.0 | Programmatic SVG document construction | The standard Rust crate for building SVG trees. Simple builder API, no dependencies on C libraries. Generates SVG strings/documents that feed directly into resvg. Mature and stable (last release Sep 2024, API settled). | HIGH |

**Rationale:** Do not hand-roll SVG string concatenation. The `svg` crate provides typed node construction (`Document`, `Element`, `Path`, `Text`, etc.) that prevents malformed output. For eidos, each animation frame becomes an SVG document, so a clean generation API is critical. The crate is lightweight (~zero transitive deps) and composes well.

**Alternative considered:** Raw `format!()` string building. Rejected because SVG attribute escaping and namespace handling become error-prone at scale.

### SVG Rasterization

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `resvg` | 0.46.0 | SVG-to-pixel rasterization | The only production-quality, pure-Rust SVG renderer. Uses tiny-skia internally. Maintained by linebender (the Rust 2D graphics organization). Handles gradients, filters, clipping, text (via fontdb). No C dependencies. | HIGH |
| `usvg` | (bundled with resvg) | SVG simplification/normalization | Companion to resvg. Parses SVG into a simplified tree, resolving styles, `use` references, etc. If eidos later needs to inspect/transform SVG before rasterizing, usvg is the tool. | HIGH |
| `tiny-skia` | 0.11.4 | 2D rasterization backend (used by resvg) | Pure-Rust Skia subset. You will not call tiny-skia directly in v1 -- resvg wraps it. Listed here because understanding the stack matters: resvg delegates all pixel work to tiny-skia. If you ever need to composite frames or draw non-SVG overlays, tiny-skia is already in your dependency tree. | HIGH |

**Rationale:** resvg is the clear winner. It is the only serious pure-Rust SVG renderer. librsvg (GNOME) requires Cairo/C dependencies. The resvg + tiny-skia combination gives you a fully Rust-native rendering pipeline with zero system library requirements. This is a massive advantage for distribution and cross-compilation.

**Pipeline:** `svg` crate builds SVG string -> `usvg` parses to tree -> `resvg` renders to `tiny_skia::Pixmap` (RGBA pixel buffer).

### Video Encoding

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `ffmpeg-sidecar` | 2.3.0 | Frame-to-video encoding via FFmpeg subprocess | Wraps the FFmpeg CLI binary as a subprocess with stdin/stdout piping. Pipe raw RGB frames in, get MP4/GIF out. No C library linking, no build complexity. Iterator-based API for frame data. | HIGH |

**Rationale -- why subprocess, not FFmpeg bindings:**

1. **Build simplicity.** FFmpeg C bindings (`ffmpeg-next`, `ffmpeg-the-third`, `rsmpeg`) require linking against libavcodec/libavformat/etc. This means users need FFmpeg development headers installed, and cross-compilation becomes painful. `ffmpeg-sidecar` only needs the `ffmpeg` binary on PATH, which most systems already have.

2. **Maintenance burden.** The FFmpeg Rust binding ecosystem has a history of abandonment: `ffmpeg` (abandoned) -> `ffmpeg-next` (abandoned) -> `ffmpeg-the-third` (current fork, v3.0.2). `ffmpeg-sidecar` sidesteps this entire chain.

3. **Performance is sufficient.** For offline video generation (not real-time), the overhead of piping raw frames through stdin is negligible. A 1920x1080 frame at 30fps is ~186 MB/s of raw data -- well within pipe throughput on any modern OS.

4. **Codec flexibility.** FFmpeg CLI supports every codec. Want H.264? H.265? VP9? AV1? GIF? Just change the CLI args. No need to conditionally compile against different FFmpeg feature flags.

**Alternatives considered:**

| Alternative | Why Not |
|-------------|---------|
| `ffmpeg-the-third` 3.0.2 | Requires FFmpeg dev headers. Build complexity not worth it for offline rendering. Fork-of-fork-of-fork maintenance risk. |
| `ffmpeg-next` 8.0.0 | Abandoned. Last real maintenance activity unclear. |
| `video-rs` 0.10.3 | Self-described as "work-in-progress". Depends on ffmpeg-next (abandoned). |
| `rsmpeg` | Requires FFmpeg dev headers. Good crate, but same build complexity issue. |
| Raw `std::process::Command` | Works, but ffmpeg-sidecar provides progress callbacks, error handling, and frame iteration for free. |

### Frame Buffer / Image Handling

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `image` | 0.25.9 | PNG encoding for individual frames, image format conversions | Standard Rust image processing crate. Needed if you want to export individual frames as PNG (debugging, thumbnails). Also provides the `RgbaImage` / `ImageBuffer` types that are standard across the ecosystem. | HIGH |
| `png` | 0.18.0 | Fast PNG encoding (optional, for performance) | Dedicated PNG encoder. Faster than `image` crate's PNG backend due to fdeflate. Consider if frame export becomes a bottleneck. | MEDIUM |

**Note:** For the core pipeline (SVG -> video), you may not need `image` at all. `resvg` renders to `tiny_skia::Pixmap` which gives you raw RGBA bytes. `ffmpeg-sidecar` consumes raw RGB bytes. The conversion is just stripping the alpha channel. But `image` is useful for debugging (save intermediate frames) and potential future PNG sequence export.

### Animation / Easing

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `simple-easing` | 1.0.2 | Easing functions (ease-in, ease-out, bezier, etc.) | Minimal, zero-dependency crate. All functions are `fn(f32) -> f32` -- trivially composable. Covers all standard easing curves (quadratic, cubic, sine, elastic, bounce, etc.). Just released 1.0 stable on Rust 2024 edition. | HIGH |

**Rationale -- why not a framework:**

Eidos will own its animation timeline system. What you need from a library is just the mathematical easing functions (the curves), not an animation framework. `simple-easing` gives you exactly that: pure functions that map `[0.0, 1.0] -> [0.0, 1.0]`.

**What to build yourself:**
- Timeline / keyframe system (core eidos domain logic)
- Interpolation between scene states (eidos-specific, depends on your object model)
- Frame scheduling (which frames to render at what time)

**Alternatives considered:**

| Alternative | Why Not |
|-------------|---------|
| `enterpolation` 0.3.0 | More powerful (bezier curves, B-splines), but heavier API. Consider adopting later if you need custom curve interpolation beyond standard easings. |
| `mina` | Designed for GUI animation loops. Brings its own timeline concept that would conflict with eidos's declarative model. |
| `bevy_tweening` | Bevy-specific. Massive unnecessary dependency. |
| `interpolation` | Older, less maintained. `simple-easing` is cleaner. |

### Math / Geometry

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `lyon` | 1.0.16 | Path tessellation and bezier curve utilities | Not for GPU rendering (eidos uses SVG/resvg), but lyon's `lyon_geom` sub-crate has excellent bezier curve math: subdivision, flattening, arc length, nearest point. Useful for smooth spline rendering and curve animation. | MEDIUM |
| `nalgebra` or `glam` | latest | Linear algebra (transforms, matrices) | For 2D transforms (translate, rotate, scale) on scene objects. `glam` is simpler and faster for 2D/3D. `nalgebra` is more general. **Recommendation: `glam`** for v1 -- lighter, faster, sufficient for 2D. | MEDIUM |

**Note on lyon:** You do not need lyon for v1. SVG path strings handle curves natively. But if you later want to programmatically manipulate bezier control points, compute arc lengths for animation timing, or do path boolean operations, `lyon_geom` is the tool. Pin this as a "phase 2" consideration.

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `rayon` | latest | Parallel frame rendering | When rendering hundreds of frames, parallelize across CPU cores. Each frame is independent (SVG -> pixels), so embarrassingly parallel. Add when performance matters. |
| `serde` + `serde_json` | latest | Scene serialization | If you want to save/load scene descriptions as JSON/TOML. Not needed for v1 core, but useful for tooling. |
| `clap` | latest | CLI argument parsing | If eidos ships a CLI tool (not just a library). Parse input file, output path, resolution, FPS, etc. |
| `tempfile` | latest | Temporary frame storage | If the pipeline needs to write intermediate frames to disk before encoding. Prefer in-memory piping, but tempfile is the fallback. |
| `fontdb` | (bundled with resvg) | Font discovery | resvg uses fontdb to find system fonts for SVG text rendering. You get this for free. |

## Existing Rust Visualization Libraries (Landscape)

Know these to differentiate, not to depend on:

| Library | What It Does | Relevance to eidos |
|---------|-------------|-------------------|
| **plotters** | Static chart rendering (PNG/SVG/HTML canvas) | Produces static images, not animations. Different problem space. eidos could eventually render plotters-style charts but with animation. |
| **charming** | ECharts wrapper for Rust (static charts) | Wraps a JavaScript engine. Wrong philosophy for eidos (pure Rust, no JS). |
| **mathlikeanim-rs** 0.10.x | Manim-inspired, targets WebAssembly/browser | Closest competitor. Targets web (Canvas/SVG in browser), not video output. Interactive focus vs eidos's offline video focus. Different niche. |
| **noon** | Manim-inspired, uses nannou/bevy/lyon | **Abandoned** (last commit Feb 2022). Used GPU rendering via nannou. Validates the idea but proves the approach needs to be simpler (noon over-engineered with bevy ECS). |

**Differentiation strategy for eidos:**
- noon died from complexity (bevy ECS for an animation lib is overkill)
- mathlikeanim-rs targets browsers, not video files
- No existing Rust library does "declarative scene -> MP4" with a clean API
- eidos's niche is clear: offline video generation, declarative API, pure Rust, statistical visualization focus

## Architecture: The Rendering Pipeline

```
User code (declarative scene description)
    |
    v
Scene graph (eidos internal representation)
    |
    v
Timeline resolution (which objects, what state, at each frame)
    |
    v
Per-frame SVG generation (svg crate)
    |
    v
SVG parsing + simplification (usvg)
    |
    v
Rasterization to RGBA pixels (resvg -> tiny-skia)
    |
    v
Strip alpha, pipe RGB to ffmpeg stdin (ffmpeg-sidecar)
    |
    v
MP4 / GIF output file
```

## What NOT to Use

| Technology | Why Not |
|------------|---------|
| `wgpu` / GPU rendering | Adds GPU dependency. Overkill for offline SVG rendering. Save for v2 if performance demands it. |
| `nannou` | Creative coding framework. Brings a window, event loop, GPU context -- none of which eidos needs for headless video generation. |
| `bevy` | Game engine. noon tried this and abandoned the project. Massive dependency for zero benefit. |
| `librsvg` | Requires Cairo (C library). Defeats the pure-Rust advantage. |
| `skia-safe` (Skia bindings) | Requires building Google Skia from source (massive C++ build). tiny-skia provides what you need without the pain. |
| `cairo-rs` | C library dependency. Same issue as librsvg. |
| `LaTeX` / `MathJax` rendering | Out of scope per PROJECT.md. Would add enormous complexity for the GAM visualization use case. |

## Installation

```bash
# Core dependencies
cargo add svg@0.18.0
cargo add resvg@0.46.0
cargo add tiny-skia@0.11.4  # Usually pulled in by resvg, but pin explicitly
cargo add ffmpeg-sidecar@2.3.0
cargo add simple-easing@1.0.2

# Useful from the start
cargo add image@0.25.9

# Add later as needed
# cargo add rayon
# cargo add lyon_geom
# cargo add glam
# cargo add clap
# cargo add serde serde_json
```

**System requirement:** FFmpeg binary on PATH. Install via:
- macOS: `brew install ffmpeg`
- Ubuntu: `apt install ffmpeg`
- Windows: Download from ffmpeg.org, add to PATH

## Version Pinning Strategy

Pin major+minor versions in Cargo.toml for the core pipeline (`resvg`, `tiny-skia`, `ffmpeg-sidecar`). These are the load-bearing dependencies. Let Cargo resolve compatible patches.

```toml
[dependencies]
svg = "0.18"
resvg = "0.46"
tiny-skia = "0.11"
ffmpeg-sidecar = "2.3"
simple-easing = "1.0"
image = "0.25"
```

## Sources

- [resvg GitHub (linebender)](https://github.com/linebender/resvg) -- SVG renderer, v0.46.0
- [resvg on crates.io](https://crates.io/crates/resvg)
- [resvg docs.rs](https://docs.rs/crate/resvg/latest) -- confirmed v0.45.1/0.46.0
- [tiny-skia GitHub (linebender)](https://github.com/linebender/tiny-skia) -- 2D rasterizer, v0.11.4
- [tiny-skia on lib.rs](https://lib.rs/crates/tiny-skia)
- [svg crate on crates.io](https://crates.io/crates/svg) -- v0.18.0
- [svg crate on lib.rs](https://lib.rs/crates/svg)
- [ffmpeg-sidecar GitHub](https://github.com/nathanbabcock/ffmpeg-sidecar) -- subprocess FFmpeg, v2.3.0
- [ffmpeg-sidecar on lib.rs](https://lib.rs/crates/ffmpeg-sidecar)
- [ffmpeg-the-third GitHub](https://github.com/shssoichiro/ffmpeg-the-third) -- FFmpeg bindings fork, v3.0.2
- [ffmpeg-next on crates.io](https://crates.io/crates/ffmpeg-next) -- v8.0.0 (abandoned)
- [video-rs on lib.rs](https://lib.rs/crates/video-rs) -- v0.10.3 (WIP)
- [simple-easing on lib.rs](https://lib.rs/crates/simple-easing) -- v1.0.2
- [enterpolation on lib.rs](https://lib.rs/crates/enterpolation) -- v0.3.0
- [mina on lib.rs](https://lib.rs/crates/mina) -- animation framework
- [image crate on crates.io](https://crates.io/crates/image) -- v0.25.9
- [png crate docs.rs](https://docs.rs/crate/png/latest) -- v0.18.0
- [plotters GitHub](https://github.com/plotters-rs/plotters) -- static chart library
- [charming GitHub](https://github.com/yuankunzhang/charming) -- ECharts wrapper
- [mathlikeanim-rs GitHub](https://github.com/MathItYT/mathlikeanim-rs) -- Manim-inspired, browser target
- [noon GitHub](https://github.com/yongkyuns/noon) -- Manim-inspired, abandoned Feb 2022
- [lyon on lib.rs](https://lib.rs/crates/lyon) -- path tessellation, v1.0.16
- [Linebender blog (resvg stewardship)](https://linebender.org/blog/tmil-14/)

---

## v1.1 Addendum: 3D Surface Mesh Rendering

**Researched:** 2026-02-25
**Scope:** What new dependencies, math, and patterns are needed to add 3D perspective mesh rendering, camera orbit animation, surface fitting animation, and scatter point visualization — while staying within the existing SVG pipeline.

### Conclusion Up Front

**One new dependency: `nalgebra = "0.34"`.**

Everything else — mesh grid generation, painter's algorithm depth sorting, SVG polygon emission, camera orbit parameterization, surface morph interpolation — is implemented in-crate with 10–30 lines of Rust per concern. No new crates beyond nalgebra are warranted.

### New Dependency

| Technology | Version | Purpose | Why Recommended | Confidence |
|------------|---------|---------|-----------------|------------|
| `nalgebra` | `0.34` | 3D linear algebra: `Point3<f64>`, `Vector3<f64>`, `Matrix4<f64>`, `Perspective3`, `Isometry3` view transforms | `Perspective3::project_point()` is the exact API needed to map 3D world coordinates to 2D NDC space. `Isometry3::look_at_rh(eye, target, up)` produces the view matrix for camera orbit. f64 native throughout — no precision loss from scientific data. Latest version 0.34.1 released Sep 20, 2025. Actively maintained by dimforge. | HIGH |

Add to `Cargo.toml`:

```toml
nalgebra = "0.34"
```

### Why nalgebra Over glam

Both nalgebra 0.34 and glam 0.32 (Feb 11, 2026) are actively maintained. The choice is nalgebra because:

1. **`Perspective3` exists as a first-class type.** `Perspective3::project_point(&point)` projects a `Point3<f64>` to NDC in one call. With glam, you construct a raw `DMat4::perspective_rh(...)` and multiply manually — achievable, but more code for zero benefit.
2. **`Isometry3::look_at_rh(eye, target, up)` exists.** Camera construction as a typed isometry, not raw matrix math.
3. **f64 is idiomatic in nalgebra.** glam's primary types are f32; f64 variants exist (`DVec3`, `DMat4`) but are secondary. For scientific visualization (GAM output data, precise axis coordinates), f64 is non-negotiable.
4. **Dimforge ecosystem coherence.** If physics simulation or constraint solving is ever added, nalgebra is the bridge.

glam would be the right choice for a game engine or f32 performance-critical path. This is neither.

### What Is NOT Needed as a Dependency

| Capability | Why No Dependency |
|------------|-------------------|
| NxM mesh grid generation | `(0..n).flat_map(|i| (0..m).map(|j| world_point(i, j)))` — 5 lines of Rust |
| Painter's algorithm depth sort | `faces.sort_by(|a, b| a.centroid_z().partial_cmp(&b.centroid_z()))` — 3 lines |
| SVG polygon / fill output | `svg::node::element::Polygon` already exists in `svg = "0.18"` |
| Camera orbit parameterization | `(azimuth, elevation, radius)` → Cartesian eye point is 4 lines of trig |
| Surface morph interpolation | Existing `Tween<P>` + `CanTween` derive handles any f64 struct. Extend `SurfaceState` to hold a `Vec<f64>` z-values grid. |
| Color-by-height mapping | Linear interpolation between two colors over a value range — in-crate |

Do not add `ndarray`, `meshgrid`, `itertools`, `interpn`, `genmesh`, or `plotters` as 3D helpers. Each adds transitive dependencies for functionality achievable in one function.

### 3D-to-SVG Rendering Architecture

The entire 3D pipeline lives within the per-frame `to_svg_elements()` call — the same pattern as existing 2D primitives. No changes to the rendering pipeline itself.

```
SurfaceMesh { vertices: Vec<Vec<Point3<f64>>>, ... }
    |
    v  [per frame, inside to_svg_elements()]
    |
Build NxM grid of Point3<f64> in world space
(or accept user-supplied grid for fitted surface)
    |
    v
nalgebra: view_matrix = Isometry3::look_at_rh(eye, target, up).to_homogeneous()
           proj_matrix = Perspective3::new(aspect, fovy, znear, zfar).as_matrix()
           mvp = proj_matrix * view_matrix
    |
    v
For each vertex: ndc = mvp * homogeneous(point)
                 screen_x = (ndc.x / ndc.w + 1.0) * width / 2.0
                 screen_y = (1.0 - ndc.y / ndc.w) * height / 2.0
    |
    v
Build quad faces: each face = 4 projected screen points + depth = centroid z in view space
    |
    v
Sort faces back-to-front by view-space z (painter's algorithm)
    |
    v
Emit SVG <polygon points="x1,y1 x2,y2 x3,y3 x4,y4"> with fill + stroke per face
    |
    v
Existing resvg rasterization pipeline — UNCHANGED
```

### Rendering Variants and SVG Approach

| Variant | SVG Element | Notes |
|---------|-------------|-------|
| Wireframe only | `<polygon fill="none" stroke="#color">` | Clean Manim aesthetic |
| Flat shaded (solid) | `<polygon fill="rgb(r,g,b)" stroke="#color">` | Per-face color from height or colormap |
| Shaded + wireframe | Two polygon layers: fill then stroke | Or single polygon with both attributes |
| Scatter points | `<circle cx="..." cy="..." r="...">` | Projected and depth-sorted with faces |

No SVG gradients or per-vertex shading needed. Flat per-face color is the correct aesthetic for a Manim-inspired library and is much simpler to implement.

### Animation Patterns

| Animation | Implementation |
|-----------|----------------|
| Surface fitting (flat → final) | `SurfaceState { z_values: Vec<f64> }` deriving `CanTween`. Each frame interpolates z_values. Exact same pattern as SplineFit. |
| Camera orbit | `CameraState { azimuth: f64, elevation: f64, radius: f64 }` deriving `CanTween`. Each frame: compute eye = spherical_to_cartesian(azimuth, elevation, radius), call look_at_rh. |
| Scatter point appearance | `ScatterPointState { x, y, z, alpha: f64 }` — tween alpha 0→1. |

All of these fit the existing `Tween<P: CanTween>` system without modification to the animation engine.

### Depth Sorting Limitation and Acceptance Criterion

The painter's algorithm with centroid depth sorting has known failure cases: highly non-convex surfaces where faces interleave. For GAM partial dependence surfaces and smooth analytical surfaces, this never occurs at reasonable mesh densities (20×20 to 100×100 quads). Acceptance criterion: no visible depth artifacts on smooth surfaces. If a future use case requires non-convex geometry, revisit with BSP tree or z-buffer composite — but that is not a v1.1 concern.

### What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `wgpu` / GPU path | Adds GPU driver dependency, breaks headless CI, forces pixel output | SVG painter's algorithm — already described |
| `glam` | f32-first design; no `Perspective3`/`Isometry3` helpers; more code for same result | `nalgebra 0.34` |
| `plotters` as 3D sub-renderer | Couples projection + styling + output; v0.3.7 (Sep 2024) API is opinionated and inflexible for eidos's aesthetic | Own the 3D-to-SVG path in-crate |
| `cgmath` | Largely unmaintained (last release 2022). No `Perspective3` analog. | `nalgebra 0.34` |
| `kiss3d` | Interactive window-based renderer; not headless, not SVG | Not applicable |
| `ndarray` for mesh grids | Vec of Vecs is sufficient; ndarray adds a large transitive dep tree for zero ergonomic gain at this scale | `Vec<Vec<nalgebra::Point3<f64>>>` |
| `fast-surface-nets` | Extracts isosurfaces from signed distance fields — wrong domain. Surface mesh from `f(x,z)->y` is just nested loops. | In-crate grid construction |

### Confirmed Versions (as of 2026-02-25)

| Crate | Latest Version | Release Date | Status |
|-------|---------------|--------------|--------|
| `nalgebra` | 0.34.1 | Sep 20, 2025 | Actively maintained |
| `glam` (considered, not chosen) | 0.32.0 | Feb 11, 2026 | Actively maintained |
| `plotters` (considered, not chosen) | 0.3.7 | Sep 8, 2024 | Active |
| `cgmath` (ruled out) | ~0.18 | 2022 | Largely unmaintained |

### v1.1 Sources

- [lib.rs/crates/nalgebra](https://lib.rs/crates/nalgebra) — version 0.34.1 (Sep 20, 2025) confirmed HIGH
- [docs.rs nalgebra Perspective3](https://docs.rs/nalgebra/latest/nalgebra/geometry/struct.Perspective3.html) — `project_point()` API confirmed HIGH
- [docs.rs nalgebra Isometry3](https://docs.rs/nalgebra/latest/nalgebra/geometry/type.Isometry3.html) — `look_at_rh()` confirmed MEDIUM (via search summary)
- [docs.rs glam DMat4](https://docs.rs/glam/latest/glam/f64/struct.DMat4.html) — `perspective_rh`, `look_at_rh` confirmed in glam 0.32.0 HIGH
- [lib.rs/crates/glam](https://lib.rs/crates/glam) — version 0.32.0 (Feb 11, 2026) confirmed HIGH
- [lib.rs/crates/plotters](https://lib.rs/crates/plotters) — version 0.3.7 (Sep 8, 2024); SVG backend and coupled architecture confirmed HIGH
- [github.com/plotters-rs 3d-plot.rs example](https://github.com/plotters-rs/plotters/blob/master/plotters/examples/3d-plot.rs) — SVGBackend usage and projection approach confirmed HIGH
- [dimforge.com rapier 2025 review](https://dimforge.com/blog/2026/01/09/the-year-2025-in-dimforge/) — nalgebra vs glam design rationale from nalgebra authors HIGH
- [bitshifter/mathbench-rs](https://github.com/bitshifter/mathbench-rs) — glam vs nalgebra performance comparison MEDIUM
