# Technology Stack

**Project:** eidos -- Manim-inspired Rust animation/visualization library
**Researched:** 2026-02-24

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
