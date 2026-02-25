# Architecture Patterns

**Domain:** Manim-inspired declarative animation library (Rust, video output)
**Researched:** 2026-02-24

## Recommended Architecture

Eidos should follow a **layered pipeline architecture** with five distinct components, mirroring Manim's proven layered design but adapted for Rust's ownership model and the SVG-to-rasterize-to-encode pipeline specified in PROJECT.md.

```
User Code (declarative scene! { } macro / builder API)
       |
       v
  +------------------+
  |   Scene Graph     |  <-- Owns tree of SceneObjects, tracks state transitions
  +------------------+
       |
       v
  +------------------+
  |  Animation Engine |  <-- Resolves transitions, interpolates per-frame state
  +------------------+
       |
       v
  +------------------+
  |   SVG Renderer    |  <-- Converts resolved scene state -> SVG document per frame
  +------------------+
       |
       v
  +------------------+
  |   Rasterizer      |  <-- SVG -> pixel buffer (resvg/tiny-skia)
  +------------------+
       |
       v
  +------------------+
  |   Video Encoder   |  <-- Pixel buffers -> MP4/GIF (ffmpeg CLI or wrapper)
  +------------------+
```

### Component Boundaries

| Component | Responsibility | Owns | Communicates With |
|-----------|---------------|------|-------------------|
| **Scene Graph** | Stores the declarative scene description: objects, their properties, and declared state transitions. Tree structure with parent-child grouping. | `SceneObject` tree, transition declarations | Animation Engine (reads transitions), User API (receives declarations) |
| **Animation Engine** | Given a scene graph with transitions, produces per-frame resolved state. Handles timing, easing, interpolation. For each frame `t`, emits a "resolved scene" where every object has concrete property values. | Timeline, easing functions, interpolation logic | Scene Graph (reads), SVG Renderer (emits resolved state) |
| **SVG Renderer** | Converts a resolved scene snapshot (all objects with concrete values) into an SVG document string. Each object type knows how to emit its SVG representation. | SVG generation logic, coordinate transforms | Animation Engine (receives resolved state), Rasterizer (emits SVG string) |
| **Rasterizer** | Takes an SVG string, produces a pixel buffer (RGBA). Thin wrapper around resvg. | resvg/tiny-skia configuration | SVG Renderer (receives SVG), Video Encoder (emits pixel buffer) |
| **Video Encoder** | Accepts a stream of pixel buffers, encodes them into MP4 or GIF. Wraps ffmpeg (CLI subprocess or FFI). | Encoding state, output file handle | Rasterizer (receives frames), Filesystem (writes output) |

### Data Flow

```
1. User writes declarative scene description
   scene! {
     axes(x: 0..10, y: 0..1)
     curve(data: spline_fit, color: blue)
       .appear()                      // transition: fade in
       .then(band(confidence: 0.95))  // transition: add confidence band
       .then(label("GAM fit"))        // transition: add label
   }

2. Scene Graph Construction
   - Macro/builder expands into SceneGraph struct
   - Tree of SceneObjects with declared transitions
   - Each transition = (trigger_order, target_state, duration, easing)

3. Animation Engine Resolution (per frame)
   - Total timeline computed from transition durations
   - For frame at time t:
     - Determine which transitions are active
     - Compute alpha (0.0..1.0) for each active transition
     - Apply easing function to alpha
     - Interpolate object properties: color, opacity, position, path points
   - Emit ResolvedScene: Vec<ResolvedObject> with concrete values

4. SVG Generation
   - Walk ResolvedScene
   - Each ResolvedObject -> SVG element(s)
   - Axes -> <line>, <text> elements
   - Curves -> <path> with Bezier control points
   - Bands -> <path> with fill-opacity
   - Compose into single SVG document string

5. Rasterization
   - Parse SVG string with resvg (usvg for parsing, tiny-skia for rendering)
   - Output: Vec<u8> RGBA pixel buffer at target resolution

6. Encoding
   - Stream pixel buffers to ffmpeg subprocess via stdin pipe
   - ffmpeg encodes to H.264 MP4 or GIF
   - Output: video file on disk
```

## Patterns to Follow

### Pattern 1: Interpolatable Trait

Every animatable property must implement a common interpolation trait. This is the core abstraction that makes the animation engine generic.

**What:** A trait that defines how to blend between two values given an alpha (0.0 to 1.0).
**When:** Any property that can change during animation.

```rust
pub trait Interpolatable: Clone {
    fn interpolate(&self, target: &Self, alpha: f64) -> Self;
}

// Implementations for common types
impl Interpolatable for f64 {
    fn interpolate(&self, target: &Self, alpha: f64) -> Self {
        self * (1.0 - alpha) + target * alpha
    }
}

impl Interpolatable for Color {
    fn interpolate(&self, target: &Self, alpha: f64) -> Self {
        Color {
            r: self.r.interpolate(&target.r, alpha),
            g: self.g.interpolate(&target.g, alpha),
            b: self.b.interpolate(&target.b, alpha),
            a: self.a.interpolate(&target.a, alpha),
        }
    }
}

// For paths (curves): interpolate control points pairwise
impl Interpolatable for BezierPath {
    fn interpolate(&self, target: &Self, alpha: f64) -> Self {
        // Requires same number of control points (resample if needed)
        BezierPath {
            points: self.points.iter().zip(&target.points)
                .map(|(a, b)| a.interpolate(b, alpha))
                .collect()
        }
    }
}
```

### Pattern 2: Transition as Data, Not Callbacks

**What:** Transitions (animations) are plain data structs, not closures or trait objects. The animation engine interprets them.
**When:** Always. This is fundamental to the declarative model.

```rust
pub enum Transition {
    FadeIn { duration: f64, easing: EasingFn },
    FadeOut { duration: f64, easing: EasingFn },
    Transform {
        target_state: ObjectState,
        duration: f64,
        easing: EasingFn,
    },
    Introduce {
        child: SceneObject,
        effect: IntroEffect,
        duration: f64,
    },
    Wait { duration: f64 },
}

pub struct Timeline {
    pub entries: Vec<TimelineEntry>,
}

pub struct TimelineEntry {
    pub object_id: ObjectId,
    pub transition: Transition,
    pub start_time: f64,  // computed from sequencing
}
```

**Why:** Data-driven transitions can be inspected, serialized, reordered, and optimized. Closures cannot.

### Pattern 3: Scene Objects as Enum, Not Trait Objects

**What:** Use an enum for scene object types rather than `Box<dyn SceneObject>`. Eidos has a known, bounded set of object types.
**When:** When the set of visual primitives is closed and known at compile time.

```rust
pub enum SceneObject {
    Axes(AxesConfig),
    Curve(CurveConfig),
    ConfidenceBand(BandConfig),
    Label(LabelConfig),
    Group(Vec<SceneObject>),
    Point(PointConfig),
    Line(LineConfig),
}
```

**Why:** Enums give exhaustive matching (compiler catches missing cases), no vtable overhead, and simpler serialization. Add a trait-object escape hatch (`Custom(Box<dyn CustomObject>)`) later if needed.

### Pattern 4: Frame Iterator Pattern

**What:** The animation engine produces frames as an iterator, decoupling frame generation from consumption.
**When:** Always. This is the natural interface between animation engine and rendering pipeline.

```rust
pub struct FrameIterator<'a> {
    scene: &'a SceneGraph,
    timeline: &'a Timeline,
    frame_rate: f64,
    current_frame: u64,
    total_frames: u64,
}

impl<'a> Iterator for FrameIterator<'a> {
    type Item = ResolvedScene;

    fn next(&mut self) -> Option<ResolvedScene> {
        if self.current_frame >= self.total_frames {
            return None;
        }
        let t = self.current_frame as f64 / self.frame_rate;
        let resolved = self.resolve_at(t);
        self.current_frame += 1;
        Some(resolved)
    }
}
```

**Why:** Iterator pattern enables lazy evaluation, easy parallelization of frame rendering, and composability with Rust's iterator adapters. The encoder can pull frames on demand rather than buffering everything in memory.

### Pattern 5: Coordinate System Abstraction

**What:** Scene objects use a logical coordinate system (e.g., data coordinates for plots). The SVG renderer maps logical coordinates to SVG viewport coordinates.
**When:** Always. Users think in data space, not pixel space.

```rust
pub struct CoordinateMap {
    pub x_range: (f64, f64),   // data space
    pub y_range: (f64, f64),   // data space
    pub viewport: Viewport,     // pixel space
}

impl CoordinateMap {
    pub fn to_svg(&self, data_point: Point) -> SvgPoint {
        // Linear mapping from data coordinates to SVG viewport
    }
}
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: Scene-Renderer Coupling (Manim's Mistake)

**What:** Manim's Scene class is tightly coupled to its Renderer -- the official docs acknowledge "there is a lot of interplay between a scene and its renderer, which is a flaw" they are working to reduce.
**Why bad:** Makes it impossible to swap rendering backends, test scene logic without rendering, or add new output formats.
**Instead:** Keep the scene graph and animation engine completely ignorant of rendering. They produce `ResolvedScene` data; the renderer consumes it. No renderer references leak into scene/animation types.

### Anti-Pattern 2: Imperative Animation Sequencing

**What:** Manim uses `self.play(FadeIn(obj))` imperatively in a `construct()` method. The scene is built by executing side effects in order.
**Why bad:** For eidos, the whole point is declarative composition. Imperative sequencing is harder to analyze, optimize, and compose. It also maps poorly to Rust (mutable self across async-like steps).
**Instead:** Transitions are declared as data on the scene graph. The animation engine resolves sequencing from the declaration order and `.then()` chaining. No imperative "play" calls.

### Anti-Pattern 3: Storing Animation State on Objects

**What:** Modifying scene objects in place during animation (Manim mutates mobjects directly).
**Why bad:** In Rust, this creates borrow checker conflicts with the scene graph. Conceptually, the "source of truth" should be the scene graph + transitions, not mutated object state.
**Instead:** The animation engine reads the scene graph immutably and produces new `ResolvedScene` snapshots. Scene objects are never mutated during rendering. This is a functional approach: `resolve(scene_graph, time) -> resolved_scene`.

### Anti-Pattern 4: String-Based SVG Assembly

**What:** Building SVG by concatenating strings manually.
**Why bad:** Injection bugs, no validation, hard to maintain, no structure.
**Instead:** Use the `svg` crate or a typed SVG builder that produces well-formed documents from structured data.

## Suggested Build Order

Components have clear dependency ordering. Build from the bottom up, validate each layer independently.

```
Phase 1: Foundation Types
  - Core types: Point, Color, BezierPath
  - Interpolatable trait + implementations
  - Easing functions (linear, ease-in-out, cubic bezier)
  - No dependencies on other components

Phase 2: Scene Graph
  - SceneObject enum (Axes, Curve, Band, Label, Group, Line, Point)
  - SceneGraph struct (tree of objects with IDs)
  - Transition enum
  - Builder API for constructing scenes programmatically
  - Depends on: Phase 1 types

Phase 3: Animation Engine
  - Timeline construction from scene graph transitions
  - Frame resolution: resolve(scene_graph, time) -> ResolvedScene
  - FrameIterator
  - Depends on: Phase 1 (interpolation), Phase 2 (scene graph)

Phase 4: SVG Renderer
  - ResolvedObject -> SVG element conversion
  - Coordinate mapping (data space -> SVG viewport)
  - Full SVG document composition
  - Depends on: Phase 1 (types), Phase 3 output (ResolvedScene)

Phase 5: Rasterizer + Encoder Pipeline
  - SVG string -> pixel buffer (resvg wrapper)
  - Pixel buffer stream -> video file (ffmpeg subprocess)
  - End-to-end: scene description -> video file
  - Depends on: Phase 4 (SVG output)

Phase 6: Declarative Macro API
  - scene! { } macro that expands to builder API calls
  - Ergonomic sugar on top of the builder
  - Depends on: Phase 2 (builder API must be stable first)

Phase 7: GAM-Specific Primitives
  - SplineFit, PartialDependence, ConfidenceBand as high-level objects
  - Built on top of Curve, Band, Axes primitives
  - Depends on: Phases 2-5 (full pipeline working)
```

**Rationale for this ordering:**
- Bottom-up: each phase is independently testable
- Phase 1 is pure math, no IO, easy to get right with property tests
- Phase 2-3 can be tested by inspecting ResolvedScene data (no rendering needed)
- Phase 4 can be tested by diffing SVG strings against expected output
- Phase 5 is integration-heavy, defer until the logic layers are solid
- Phase 6 (macros) should wrap a stable API, not drive its design
- Phase 7 is domain-specific, requires the full pipeline

## Scalability Considerations

| Concern | Eidos v1 (personal tooling) | Future (general-purpose) |
|---------|----------------------------|--------------------------|
| Frame count | 30fps x 30s = 900 frames, sequential is fine | Parallelize frame rendering with rayon |
| Object count | <50 objects per scene | Spatial partitioning, dirty-region tracking |
| SVG complexity | Simple paths, resvg handles it | Consider direct tiny-skia rendering (skip SVG string) |
| Memory | Hold one frame in memory at a time (iterator pattern) | Same -- iterator pattern scales |
| Video size | 1080p MP4, ffmpeg handles it | Same -- ffmpeg scales |

## Key Architectural Decisions

### Why SVG as Intermediate Representation

The SVG-per-frame approach has a crucial advantage for v1: **debuggability**. You can open any frame's SVG in a browser and inspect it visually. This makes the rendering pipeline transparent. The cost is string serialization/parsing overhead per frame, which is negligible at 900 frames.

If performance becomes an issue later, the architecture supports dropping SVG entirely: the SVG Renderer can be replaced with a "direct tiny-skia renderer" that writes to pixel buffers directly from ResolvedScene, without changing any upstream components.

### Why Not an ECS (Entity Component System)

The `noon` project used Bevy ECS for state management. For eidos, this is overkill:
- Eidos scenes have <50 objects, not thousands
- The declarative model means no dynamic entity creation during animation
- ECS adds significant dependency weight (Bevy) and conceptual overhead
- A simple tree (Vec-based scene graph) is sufficient and more debuggable

### Why Enum Over Trait Objects for Scene Objects

Eidos has a closed set of visual primitives. Using an enum:
- Enables exhaustive pattern matching (compiler catches missing renderers)
- Avoids `Box<dyn>` heap allocation
- Makes serialization trivial (derive Serialize)
- Can add `Custom(Box<dyn ...>)` variant later for extensibility

## Sources

- [Manim Deep Dive (official internals documentation)](https://docs.manim.community/en/stable/guides/deep_dive.html) - HIGH confidence
- [Manim Community Architecture Overview (DeepWiki)](https://deepwiki.com/ManimCommunity/manim) - MEDIUM confidence
- [noon: Rust Manim-inspired animation engine](https://github.com/yongkyuns/noon) - MEDIUM confidence (unmaintained but architecturally informative)
- [mathlikeanim-rs: Rust math animation library](https://github.com/MathItYT/mathlikeanim-rs) - MEDIUM confidence
- [tiny-skia: CPU 2D rendering](https://github.com/linebender/tiny-skia) - HIGH confidence
- [resvg: SVG rendering library](https://github.com/linebender/resvg) - HIGH confidence
- [svg crate: SVG generation](https://crates.io/crates/svg) - HIGH confidence
- [tween crate: easing functions](https://docs.rs/tween) - MEDIUM confidence
