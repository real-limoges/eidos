# Feature Landscape

**Domain:** Manim-inspired Rust animation library for statistical/data visualization
**Researched:** 2026-02-24

## Table Stakes

Features users expect from a programmatic animation library. Missing any of these and the library feels broken or unusable.

### Scene & Composition

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Scene as top-level container | Every animation library (Manim, Motion Canvas, Vizzu) uses a scene abstraction. Users need a root context to add objects and sequence animations. | Low | Builder pattern or `scene!{}` macro. Manim uses `Scene.construct()`, eidos should use declarative description. |
| Object grouping / layering | Users need to treat multiple objects as one unit (e.g., an axis + labels + curve = a "plot"). Manim has `VGroup`, Motion Canvas has scene hierarchy. | Medium | `VGroup`-equivalent. Must support nested groups with relative positioning. |
| Z-ordering / draw order | Objects must layer predictably. A shaded confidence band must render behind the curve line. | Low | Implicit via add-order or explicit z-index. |
| Coordinate system / viewport | Users think in data coordinates (x: 0..100, y: -2..2), not pixel coordinates. The library must map between data space and screen space. | Medium | Manim centers at (0,0) with configurable range. eidos needs axes that define the mapping. |

### Animation Primitives

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Object creation animations | "Draw" / "Write" / "FadeIn" -- the bread and butter. Without animated appearance, it is just a static renderer. | Medium | Manim: `Create`, `Write`, `FadeIn`, `DrawBorderThenFill`. eidos needs at minimum: fade-in, draw-along-path, and instant appear. |
| Property interpolation | Animate any numeric property smoothly: position, opacity, color, scale, rotation. This is the core of "smooth animation". | Medium | Must support interpolation of position (Vec2), color (RGBA), opacity (f64), scale (f64), and arbitrary numeric values. |
| Easing / rate functions | Users expect smooth-start, smooth-end, bounce, elastic, etc. Linear-only animation looks robotic. | Low | Standard set: linear, ease-in, ease-out, ease-in-out, smooth (Manim default). Cubic bezier for custom. ~15 built-in functions covers it. |
| Transform / morph between objects | Morphing one shape into another (e.g., a bar chart morphing into a line chart, or one curve evolving into another). Manim's `Transform` and `ReplacementTransform` are heavily used. | High | Requires path interpolation between different geometries. Start with same-point-count interpolation, defer arbitrary morphing. |
| Sequential and parallel composition | Play animations one after another, or simultaneously. Manim: `AnimationGroup`, `Succession`, `LaggedStart`. | Medium | Users must be able to say "fade in the axis, then draw the curve, then shade the band" vs "draw all three at once". |
| Wait / pause | Hold the frame for N seconds between animations. Trivial but essential for pacing. | Low | Just emit duplicate frames. |

### Visual Objects

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Axes (2D cartesian) | The foundation of any data visualization. Must support tick marks, labels, configurable range, and grid lines. | High | Manim has `Axes`, `NumberPlane`. eidos needs Axes with: configurable x/y range, tick spacing, tick labels, axis labels, optional grid. This is the single most complex table-stakes object. |
| Line / curve from data points | Plot a smooth curve through data points. This is `FunctionGraph` in Manim, but eidos needs it from discrete data (not just f(x)). | Medium | Accept `Vec<(f64, f64)>` and interpolate with cubic spline or similar. Must render as smooth SVG path, not polyline. |
| Basic shapes | Circle, Rectangle, Line, Arrow, Polygon. Building blocks for annotations and diagrams. | Low | Standard SVG primitives. Parameterized with position, size, color, stroke, fill. |
| Text labels | Axis labels, titles, annotations. Must be positionable relative to other objects. | Medium | SVG text rendering. Must support font size, color, alignment, and relative positioning (`next_to`, `above`, `below`). No LaTeX needed (per PROJECT.md). |
| Color and styling | Fill color, stroke color, stroke width, opacity. Must be animatable. | Low | RGBA color model. Named color constants (RED, BLUE, etc.) for convenience. |

### Rendering Pipeline

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Video output (MP4) | The primary output format. Users run code, get a video file. | Medium | SVG-per-frame -> rasterize (resvg/tiny-skia) -> encode (ffmpeg). This is the core pipeline from PROJECT.md. |
| GIF output | For embedding in slides, READMEs, Slack. Lower quality but more portable than MP4. | Low | Same pipeline, different encoder output. ffmpeg handles this. |
| Configurable resolution & framerate | Quality presets like Manim's `-ql` (480p/15fps) for dev, `-qh` (1080p/60fps) for production. | Low | Config struct with presets. Default to 1080p/30fps. |
| Deterministic output | Same code = same video, byte-for-byte. Essential for CI, testing, reproducibility. | Low | SVG pipeline is inherently deterministic. Avoid system-font fallbacks (embed or specify). |

### Developer Experience

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Builder / declarative API | The whole point of eidos. Users describe what they want, not how to render frame 47. | High | `scene!{}` macro or builder chain. This IS the product differentiator that must ship from day one. |
| Sensible defaults | Objects should look good without configuration. Manim's defaults (white on black, smooth easing, clean vector look) are a big part of its appeal. | Medium | Default color palette, default easing, default axis styling. "Zero-config beautiful" is a design goal, not a nice-to-have. |
| Error messages that help | Rust's type system helps, but runtime errors (e.g., data range mismatch) need clear messages. | Low | Validate inputs eagerly. "x_range (0..10) doesn't contain data point x=15" not "index out of bounds". |

## Differentiators

Features that set eidos apart. Not expected in a generic animation library, but high value for the target use case.

### Statistical / Data Visualization Primitives

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Confidence bands (shaded region between curves) | **The killer feature for GAM visualization.** Manim has no built-in `fill_between` (open issue #1063 on 3b1b/manim). matplotlib's `fill_between` is the go-to but is static. Animated confidence bands growing/shrinking are novel. | Medium | Accept upper/lower curve data, render as filled SVG path with alpha. Must animate smoothly (band widening/narrowing). |
| Spline fit animation | Show a spline curve being fit to data -- the curve "settling" onto the data points. Specific to GAM/statistical use cases. No existing tool does this well in animation. | High | Animate spline parameters over time: from flat line to fitted curve. Requires interpolating between spline control points. |
| Partial dependence plot (PDP) primitives | First-class PDP: axes labeled with feature name, smooth effect curve, confidence band, optional rug plot. One function call, not 50 lines of composition. | High | Composes Axes + Curve + ConfidenceBand + RugPlot + Labels into a single high-level object. This is the "just works" experience for GAM users. |
| Data-driven object construction | Build visual objects directly from `Vec<f64>` / `Vec<(f64, f64)>` data. No manual coordinate conversion. | Medium | Axes auto-range from data. Curves built from data points. This is how data scientists think -- not in screen coordinates. |
| Rug plot | Small tick marks along an axis showing data density. Standard in GAM plots, absent from animation libraries. | Low | Short vertical lines at each data x-value along the x-axis. Simple but important for statistical context. |

### Declarative Scene Model

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| "Describe what, not when" API | Manim is imperative (`self.play(Create(circle))`). Motion Canvas uses generators. eidos's declarative model ("here are my objects and their states") with automatic animation is genuinely different. | High | The library infers transitions: "object wasn't there, now it is = fade in. Object moved = animate movement." This is the core design philosophy from PROJECT.md. |
| Automatic animation inference | Given state A and state B of a scene, automatically generate smooth transitions. User doesn't specify `FadeIn` -- they just add an object to a later state. | High | Diff two scene states, generate appropriate animations. Novel approach vs imperative sequencing. |
| Scene keyframes / states | Define the scene at discrete points ("keyframe 1: axes only", "keyframe 2: add curve", "keyframe 3: add confidence band"). Library interpolates between them. | Medium | Natural extension of declarative model. Each keyframe is a complete scene description. |

### Performance & Rust Advantages

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Fast rendering | Rust + tiny-skia should be significantly faster than Python Manim for frame generation. A 60-second video at 30fps = 1800 frames. | Low | Inherent to the language choice. Parallelize frame rendering with rayon. |
| Single binary distribution | `cargo install eidos` and done. No Python env, no system deps (except ffmpeg). Manim's installation is notoriously painful. | Low | Static linking. Bundle what you can. ffmpeg is the only external dep. |
| Type-safe scene construction | Rust's type system prevents invalid scenes at compile time. Can't accidentally pass a color where a position is expected. | Low | Newtype wrappers: `Position(f64, f64)`, `Color(u8, u8, u8, u8)`, `Opacity(f64)`. |

## Anti-Features

Features to explicitly NOT build. These are scope traps or philosophical mismatches.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| LaTeX rendering | PROJECT.md explicitly excludes it. Adds massive complexity (system deps, font handling). GAM visualizations don't need equations. | SVG text with good font support. If someone needs math notation, they can use an SVG of pre-rendered LaTeX. |
| Interactive / real-time output | Fundamentally different architecture (event loop, input handling, redraw). Doubles the API surface. Video-first is the design constraint. | Output MP4/GIF. If interactive is ever needed, it's a v2+ concern and likely a separate crate. |
| GUI / visual editor | Code-only is a design principle, not a limitation. A GUI would be a separate product. | Fast iteration via quick re-render. Preview via `--open` flag that launches the video after rendering. |
| Python bindings / PyO3 | The whole point is a Rust-native experience. Python bindings would compromise the API design (can't use macros, builders don't translate well). | Users who want Python should use Manim. eidos is for Rust users. |
| 3D rendering | Massive complexity for minimal value in the statistical visualization domain. GAM plots are 2D. | 2D only. If 3D surface plots are ever needed, evaluate as a separate module with explicit opt-in. |
| Browser / WASM output | Different rendering pipeline, different constraints. Splits focus. | Video files only for v1. WASM could be a future target but must not influence v1 architecture. |
| General-purpose charting library | eidos is not plotters or matplotlib. It's an animation engine that happens to visualize data. Don't build 47 chart types. | Focus on the primitives that compose into charts: axes, curves, bands, labels. Users compose; eidos animates. |
| Audio synchronization | Motion Canvas has this. It's a video production feature, not a data visualization feature. | Silent video output. Users can add audio in post-production if needed. |

## Feature Dependencies

```
Coordinate System (Axes) --> Curve (needs axes for data-to-screen mapping)
Coordinate System (Axes) --> Confidence Band (needs axes)
Coordinate System (Axes) --> Rug Plot (needs axes)
Curve --> Spline Fit Animation (animates curve construction)
Curve + Confidence Band --> PDP Primitive (composes both)

Property Interpolation --> All Animations (core mechanism)
Easing Functions --> Property Interpolation (modifies interpolation rate)
Sequential/Parallel Composition --> Scene Keyframes (sequences keyframe transitions)

SVG Generation --> Rasterization (resvg/tiny-skia) --> Video Encoding (ffmpeg)
All Visual Objects --> SVG Generation (everything renders to SVG)

Builder/Declarative API --> Scene Keyframes --> Automatic Animation Inference
Object Grouping --> PDP Primitive (PDP is a group of sub-objects)
```

Key dependency chain for MVP:
```
Color/Styling -> Basic Shapes -> SVG Generation -> Rasterization -> Video Encoding
                                      |
Property Interpolation + Easing -> Animation Engine -> Scene Container
                                      |
Coordinate System (Axes) -> Curve -> Confidence Band
```

## MVP Recommendation

**Prioritize (Phase 1 -- the "hello world" that's actually useful):**

1. **SVG rendering pipeline** -- without output, nothing else matters. SVG -> rasterize -> MP4.
2. **Basic shapes + color/styling** -- Circle, Rect, Line, Arrow with fill/stroke/opacity.
3. **Property interpolation + easing** -- the animation core. Interpolate position, color, opacity.
4. **Scene container with sequential animation** -- add objects, play animations, produce video.
5. **Axes (2D cartesian)** -- the foundation for any data plot. Tick marks, labels, range.
6. **Curve from data points** -- plot a smooth line through `Vec<(f64, f64)>`.

**Prioritize (Phase 2 -- the GAM visualization payoff):**

7. **Confidence bands** -- shaded fill between upper/lower curves. The differentiator.
8. **Declarative scene model** -- keyframe-based "describe states, infer animations".
9. **Spline fit animation** -- animate a curve settling onto data.
10. **PDP composite object** -- one-call partial dependence plot.

**Defer:**

- **Transform/morph** between arbitrary objects: High complexity, not needed for GAM plots. Phase 3+.
- **Rug plot**: Low complexity but low priority. Easy to add once axes exist.
- **Automatic animation inference**: The "magic" declarative feature. High complexity, needs the manual animation system to be solid first. Phase 3+.
- **Text beyond labels**: Rich text, multi-line, markdown rendering. Not needed for v1.

## Sources

- [Manim Community Docs -- Building Blocks](https://docs.manim.community/en/stable/tutorials/building_blocks.html)
- [Manim Community Docs -- Rate Functions](https://docs.manim.community/en/stable/reference/manim.utils.rate_functions.html)
- [ManimCommunity/manim DeepWiki](https://deepwiki.com/ManimCommunity/manim) -- Architecture and rendering pipeline
- [3b1b/manim Issue #1063](https://github.com/3b1b/manim/issues/1063) -- fill_between for uncertainty visualization (open issue, no built-in support)
- [noon (Rust Manim-inspired, archived)](https://github.com/yongkyuns/noon) -- Prior art for Rust animation API
- [mathlikeanim-rs](https://github.com/MathItYT/mathlikeanim-rs) -- Rust mathematical animation library
- [Motion Canvas](https://motioncanvas.io/) -- TypeScript alternative with generator-based scene composition
- [Vizzu](https://github.com/vizzuhq/vizzu-lib) -- Animated data visualization with Grammar of Graphics inspiration
- [mgcv plot.gam](https://rdrr.io/cran/mgcv/man/plot.gam.html) -- Standard GAM visualization in R (confidence bands, spline fits)
- [resvg](https://github.com/linebender/resvg) -- SVG rendering library for Rust
- [Fundamentals of Data Visualization -- Visualizing Uncertainty](https://clauswilke.com/dataviz/visualizing-uncertainty.html) -- Animated uncertainty visualization techniques
