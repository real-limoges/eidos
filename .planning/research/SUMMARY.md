# Project Research Summary

**Project:** eidos -- Manim-inspired Rust animation/visualization library
**Domain:** Programmatic animation library for statistical data visualization (GAM focus)
**Researched:** 2026-02-24
**Confidence:** HIGH

## Executive Summary

Eidos is a Rust library for generating animated statistical visualizations as video files, inspired by Manim but with a declarative API and a focus on GAM (Generalized Additive Model) plots. The expert approach for this kind of tool is a layered pipeline: declarative scene description -> scene graph -> per-frame state resolution -> SVG generation -> rasterization -> video encoding. The Rust ecosystem has mature, high-confidence crates for every stage of this pipeline: `svg` for generation, `resvg`/`tiny-skia` for rasterization (pure Rust, no C deps), and `ffmpeg-sidecar` for encoding via subprocess. No competitor occupies this niche -- `noon` (Rust Manim clone) died from over-engineering with Bevy ECS, `mathlikeanim-rs` targets browsers, and no existing Rust library does "declarative scene -> MP4" with a clean API.

The recommended approach is to build bottom-up: foundation types and interpolation first, then scene graph with builder API (not macros), then animation engine, then SVG rendering, then video encoding. The builder-first API strategy is critical -- a `scene!{}` macro should only be added once the underlying API is stable. The architecture should use arena-based ownership (slotmap) for the scene graph and enum dispatch for scene object types, avoiding trait objects in the hot path. SVG serves as the intermediate representation in v1 for debuggability, with a planned migration to direct tiny-skia rendering if performance demands it.

The top risks are: (1) scene graph ownership fights with the borrow checker -- mitigated by using arena allocation from day one; (2) premature abstraction of the rendering backend -- mitigated by committing to a single concrete SVG pipeline for v1; (3) SVG-per-frame performance ceiling at scale -- mitigated by designing the architecture so the SVG renderer is swappable without touching upstream components. The GAM visualization use case provides natural scope containment: every feature decision should be gated by "does this help render an animated GAM plot?"

## Key Findings

### Recommended Stack

The stack is pure Rust with zero C dependencies (except FFmpeg binary on PATH). Every core crate is mature, actively maintained, and high-confidence.

**Core technologies:**
- **`svg` 0.18**: SVG document construction -- typed node builder, prevents malformed output
- **`resvg` 0.46 + `tiny-skia` 0.11**: SVG rasterization -- only production-quality pure-Rust SVG renderer, maintained by linebender
- **`ffmpeg-sidecar` 2.3**: Video encoding via FFmpeg subprocess -- avoids all C linking complexity, codec-flexible
- **`simple-easing` 1.0**: Easing functions -- pure `fn(f32) -> f32`, zero deps, covers all standard curves
- **`image` 0.25**: Frame debugging/export -- standard ecosystem type (`RgbaImage`)

**Defer to later phases:**
- `lyon_geom` -- bezier curve math, only if programmatic path manipulation is needed
- `glam` -- linear algebra for transforms, only when 2D transform composition gets complex
- `rayon` -- parallel frame rendering, only when sequential rendering is too slow

**System requirement:** FFmpeg binary on PATH (ubiquitous, easy to install).

### Expected Features

**Must have (table stakes):**
- Scene container with sequential/parallel animation composition
- Basic shapes (circle, rect, line, arrow) with fill/stroke/opacity
- Property interpolation with easing (position, color, opacity, scale)
- 2D cartesian axes with ticks, labels, configurable range
- Curve from data points (`Vec<(f64, f64)>`) as smooth SVG path
- Video output (MP4/GIF) with configurable resolution and framerate
- Builder/declarative API with sensible defaults ("zero-config beautiful")

**Should have (differentiators):**
- Confidence bands (shaded fill between curves) -- the killer GAM feature, Manim lacks this
- Spline fit animation (curve "settling" onto data)
- Partial dependence plot as a single composite object
- Data-driven construction (auto-range axes from data, build from `Vec<f64>`)
- Scene keyframes with automatic transition inference

**Defer (v2+):**
- Transform/morph between arbitrary objects (high complexity, not needed for GAM)
- Automatic animation inference (needs manual system to be solid first)
- `scene!{}` macro (add only after builder API is stable)
- Rich text, LaTeX, 3D, interactive output, WASM, Python bindings

### Architecture Approach

A five-layer pipeline with strict unidirectional data flow: Scene Graph -> Animation Engine -> SVG Renderer -> Rasterizer -> Video Encoder. Each layer is independently testable. The scene graph is immutable during rendering -- the animation engine produces new `ResolvedScene` snapshots per frame (functional approach). Frame generation uses the iterator pattern for lazy evaluation and future parallelization.

**Major components:**
1. **Scene Graph** -- owns tree of SceneObjects (arena-allocated), stores transitions as data (not callbacks)
2. **Animation Engine** -- resolves per-frame state via interpolation + easing, produces `ResolvedScene`
3. **SVG Renderer** -- converts resolved state to SVG document, handles coordinate mapping (data space -> viewport)
4. **Rasterizer** -- thin wrapper around resvg, SVG string -> RGBA pixel buffer
5. **Video Encoder** -- pipes pixel buffers to ffmpeg subprocess, outputs MP4/GIF

**Key patterns:**
- `Interpolatable` trait for all animatable properties
- Transitions as enum data, not closures
- SceneObject as enum (not trait objects) for exhaustive matching and zero vtable overhead
- Frame iterator pattern decouples generation from consumption
- Single canonical coordinate system (y-up, origin center) with one Camera transform point

### Critical Pitfalls

1. **Scene graph ownership hell** -- Use `slotmap::SlotMap` arena allocation from day one. Never use `Rc<RefCell<>>` for scene graph nodes. This is the most common Rust graph structure mistake and causes full rewrites.
2. **Macro DSL trap** -- Start with builder pattern API, not `scene!{}` macro. Macros over 50 lines become unmaintainable with incomprehensible errors. Add macro sugar only after builder API is frozen (Phase 6+).
3. **Premature rendering abstraction** -- No `trait Renderer`. Build one concrete SVG pipeline. Extract abstractions only when actually adding a second backend.
4. **SVG-per-frame performance wall** -- Acceptable for v1 prototyping but will hit a ceiling at scale (thousands of frames). Architecture must support swapping in direct tiny-skia rendering without changing upstream components.
5. **Coordinate system confusion** -- Define canonical world space (y-up, center origin) immediately. All coordinate transforms happen in exactly one place (Camera/viewport module).

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation Types and Rendering Pipeline
**Rationale:** Bottom-up: nothing works without core types and the ability to produce output. Get a colored rectangle on screen in week 1.
**Delivers:** Core types (Point, Color, BezierPath), `Interpolatable` trait, easing functions, basic shapes, SVG generation, rasterization, video encoding -- a working end-to-end pipeline that renders static scenes to MP4.
**Addresses:** SVG rendering pipeline, basic shapes + styling, configurable resolution/framerate, deterministic output.
**Avoids:** Scene graph ownership hell (use arena from start), coordinate confusion (define world space immediately), FFmpeg linking hell (use ffmpeg-sidecar), no visual testing (render frames from day one).

### Phase 2: Scene Graph and Animation Engine
**Rationale:** With output working, build the core domain model. Scene graph + transitions + interpolation = animated output.
**Delivers:** SceneObject enum, SceneGraph with arena storage, Transition data types, Timeline, FrameIterator, sequential/parallel composition, property interpolation (position, color, opacity). First animated videos.
**Addresses:** Scene container, object grouping/layering, z-ordering, animation primitives (fade-in, property interpolation, easing, wait/pause), sequential/parallel composition.
**Avoids:** Trait object proliferation (enum dispatch), interpolation generality trap (build concrete FadeIn/Transform first), over-engineered timeline (sequential + parallel only).

### Phase 3: Data Visualization Primitives
**Rationale:** Axes and curves are the foundation for all GAM plots. These depend on the scene graph and animation engine being solid.
**Delivers:** 2D cartesian axes (ticks, labels, range, grid), curve from data points, coordinate system mapping (data space -> viewport), data-driven construction (auto-range from data).
**Addresses:** Axes, curve from data, coordinate system/viewport, text labels, data-driven object construction.
**Avoids:** Text rendering rabbit hole (basic labels only), scope creep (GAM use case gates features).

### Phase 4: Statistical Visualization Features
**Rationale:** The GAM differentiators. Requires axes + curves from Phase 3.
**Delivers:** Confidence bands, rug plots, spline fit animation, partial dependence plot composite object. This is the payoff phase -- the features no other tool provides.
**Addresses:** Confidence bands, spline fit animation, PDP primitive, rug plot.
**Avoids:** Scope creep toward general-purpose graphics.

### Phase 5: Declarative API and Polish
**Rationale:** Builder API must be stable before adding macro sugar. Performance optimization is deferred until the feature set is complete.
**Delivers:** `scene!{}` macro (thin veneer over builder), scene keyframes, quality presets, parallel frame rendering (rayon), potential migration from SVG to direct tiny-skia if benchmarks warrant.
**Addresses:** Declarative scene model, scene keyframes, fast rendering, single binary distribution.
**Avoids:** Macro DSL trap (builder is already proven), premature rendering abstraction (only optimize if benchmarks show need).

### Phase Ordering Rationale

- **Bottom-up validation:** Each phase produces testable output. Phase 1 renders static frames. Phase 2 animates them. Phase 3 adds data-aware objects. Phase 4 adds statistical features. Phase 5 polishes the API.
- **Dependency chain:** Types -> Scene Graph -> Animation -> SVG Rendering -> Video Encoding is the strict dependency order. Visualization primitives (Axes, Curve) depend on the scene graph. Statistical features (confidence bands) depend on visualization primitives.
- **Risk front-loading:** The hardest architectural decisions (arena-based scene graph, enum dispatch, coordinate system) are settled in Phases 1-2. Later phases build on a solid foundation.
- **Pitfall avoidance:** Builder-before-macro (Pitfall 2), concrete-before-abstract (Pitfall 3), and GAM-scope-gating (Pitfall 10) are enforced by phase ordering.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (Animation Engine):** Timeline resolution and transition sequencing have nuance. Research how Manim's Animation/AnimationGroup timing model works in detail before designing the Timeline struct.
- **Phase 3 (Data Viz Primitives):** Axes construction is the most complex table-stakes feature. Research SVG tick/label layout strategies and data-to-viewport coordinate mapping patterns.
- **Phase 4 (Statistical Features):** Spline fit animation requires interpolating between spline control points over time. Research cubic spline representation and how to animate parameter changes smoothly.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Foundation):** Well-documented patterns. Arena allocation, SVG generation, resvg usage, ffmpeg piping all have clear examples.
- **Phase 5 (Declarative API):** Proc macro patterns in Rust are well-documented. Builder-to-macro wrapping is a standard technique.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crates verified on crates.io/lib.rs with recent releases. resvg/tiny-skia are linebender-maintained. ffmpeg-sidecar solves a well-known problem. No speculative choices. |
| Features | HIGH | Feature set derived from Manim (proven), GAM visualization requirements (domain-specific), and competitive analysis of existing Rust libraries. Clear table-stakes vs differentiator separation. |
| Architecture | HIGH | Layered pipeline is the established pattern (Manim uses it). Arena-based scene graph is standard Rust practice. Enum dispatch over trait objects is well-validated. |
| Pitfalls | HIGH | Every critical pitfall is supported by prior art (noon's failure, Manim's renderer coupling, arena allocation best practices). SVG performance wall is MEDIUM confidence (reasoning-based, no benchmarks). |

**Overall confidence:** HIGH

### Gaps to Address

- **SVG-per-frame performance:** No benchmarks exist for the exact pipeline (svg crate -> resvg -> ffmpeg pipe). Benchmark early in Phase 1 to validate feasibility and establish baseline for Phase 5 optimization decisions.
- **Spline interpolation specifics:** How exactly to animate a spline "settling" onto data needs prototyping. Multiple approaches (interpolate control points, interpolate knot weights, animate degree of fit). Resolve during Phase 4 planning.
- **Font handling in resvg:** resvg uses fontdb for system font discovery. Cross-platform font consistency (macOS vs Linux) may require bundling a font. Test early in Phase 3 when labels are introduced.
- **glam vs manual transforms:** Whether glam is needed for 2D transforms or if simple `(f64, f64)` math suffices. Defer decision until Phase 2 reveals actual transform complexity.

## Sources

### Primary (HIGH confidence)
- [resvg GitHub (linebender)](https://github.com/linebender/resvg) -- SVG renderer, v0.46.0
- [tiny-skia GitHub (linebender)](https://github.com/linebender/tiny-skia) -- 2D rasterizer, v0.11.4
- [ffmpeg-sidecar GitHub](https://github.com/nathanbabcock/ffmpeg-sidecar) -- subprocess FFmpeg, v2.3.0
- [svg crate on crates.io](https://crates.io/crates/svg) -- v0.18.0
- [simple-easing on lib.rs](https://lib.rs/crates/simple-easing) -- v1.0.2
- [Manim Community Docs](https://docs.manim.community/en/stable/) -- architecture deep dive, building blocks, rate functions
- [slotmap crate documentation](https://docs.rs/slotmap/latest/slotmap/index.html) -- arena allocation
- [Builder pattern in Rust - LogRocket](https://blog.logrocket.com/build-rust-api-builder-pattern/)

### Secondary (MEDIUM confidence)
- [ManimCommunity/manim DeepWiki](https://deepwiki.com/ManimCommunity/manim) -- architecture analysis, renderer coupling issues
- [noon GitHub (archived)](https://github.com/yongkyuns/noon) -- Rust Manim attempt, failed due to Bevy ECS complexity
- [mathlikeanim-rs GitHub](https://github.com/MathItYT/mathlikeanim-rs) -- Rust math animation, browser-targeted
- [Motion Canvas](https://motioncanvas.io/) -- TypeScript alternative, generator-based composition
- [Fundamentals of Data Visualization (Wilke)](https://clauswilke.com/dataviz/visualizing-uncertainty.html) -- uncertainty visualization techniques
- [Arenas in Rust - Manish Goregaokar](https://manishearth.github.io/blog/2021/03/15/arenas-in-rust/)

### Tertiary (LOW confidence)
- [3b1b/manim Issue #1063](https://github.com/3b1b/manim/issues/1063) -- fill_between for uncertainty (validates gap in ecosystem)
- [scene-graph crate](https://docs.rs/scene-graph) -- thunderdome-backed, limited documentation

---
*Research completed: 2026-02-24*
*Ready for roadmap: yes*
