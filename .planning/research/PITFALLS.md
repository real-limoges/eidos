# Domain Pitfalls

**Domain:** Manim-inspired Rust animation/visualization library
**Project:** eidos
**Researched:** 2026-02-24

## Critical Pitfalls

Mistakes that cause rewrites or major issues.

### Pitfall 1: Scene Graph Ownership Hell

**What goes wrong:** Attempting to model a scene graph with parent-child references using `&` borrows or `Rc<RefCell<T>>`. Parent nodes need mutable access to children, children sometimes need to reference parents, and animations need to mutate objects that are borrowed elsewhere. This leads to either borrow checker fights, runtime panics from `RefCell`, or reference cycle memory leaks.

**Why it happens:** The natural mental model for a scene graph is a tree with bidirectional references (parent/child). Rust's ownership model fundamentally conflicts with shared mutable graph structures. Developers coming from Python/C++ instinctively reach for references.

**Consequences:** Weeks lost fighting the borrow checker. Code becomes riddled with `Rc<RefCell<>>` or `unsafe` blocks. Runtime panics replace compile-time safety. Eventually requires a full rewrite of the core data structure.

**Prevention:**
- Use an arena-based approach from day one. `slotmap::SlotMap` or `thunderdome::Arena` store objects in a flat collection with generational indices (keys) instead of references.
- Scene graph nodes hold `SlotMapKey` handles to parent/children, not references. All access goes through the arena.
- This is the established pattern in Rust game engines (Bevy, Fyrox) and the `scene-graph` crate uses `thunderdome` for exactly this reason.
- Design the `Scene` struct to own the arena, and pass `&mut Scene` to operations that need to modify objects.

**Detection:** If you find yourself writing `Rc<RefCell<dyn Mobject>>` or adding lifetime parameters to scene graph node types, stop immediately.

**Phase relevance:** Phase 1 (core data model). Must be correct before anything else is built on top.

**Confidence:** HIGH -- this is the most well-documented Rust graph structure challenge, with extensive community writing (Manish Goregaokar's arena blog post, nrc's graphs tutorial, Catherine West's RustConf 2018 talk on ECS).

---

### Pitfall 2: Declarative Macro DSL That Becomes Unmaintainable

**What goes wrong:** Building a `scene! { }` macro that tries to express the full scene composition language in `macro_rules!`. The macro grows to handle nested objects, property overrides, animation hints, conditional elements, and grouping. It becomes a parser within a parser -- hundreds of lines of match arms with inscrutable error messages when users make typos.

**Why it happens:** Declarative macros feel like the "Rusty" way to build DSLs, and the `scene! { }` syntax in PROJECT.md is appealing. But `macro_rules!` has fundamental limitations: no custom identifiers, limited lookahead, poor error messages, and rightward drift for nested structures.

**Consequences:** Users get incomprehensible compiler errors like "no rules expected the token `(`". Macro maintenance becomes the bottleneck. Adding new scene element types requires modifying the macro, creating tight coupling. Proc macros are an alternative but add compile-time overhead (syn/quote dependencies) and their own complexity.

**Prevention:**
- Start with a builder pattern API, not a macro. Builders compose well with Rust's type system, give clear error messages, and support IDE autocomplete.
- Example: `Scene::new().add(Axes::new().x_range(0.0, 10.0)).add(Curve::from_fn(|x| x.sin()))` is more maintainable than any macro.
- If a macro is eventually desired for ergonomics, add it as a thin veneer over the builder API in a later phase, once the underlying API is stable.
- A proc macro can be justified later if the builder becomes too verbose, but only after the API surface is frozen.

**Detection:** If your macro definition exceeds 50 lines, or if you're writing "helper macros" to decompose the main macro, the approach is not scaling.

**Phase relevance:** Phase 1-2 (API design). Get the builder API right first. Macro sugar is Phase 3+ at earliest.

**Confidence:** HIGH -- well-documented Rust ecosystem pattern. The LogRocket builder pattern guide and community consensus favor builders over complex macros for domain APIs.

---

### Pitfall 3: Premature Rendering Abstraction (Multiple Backends Too Early)

**What goes wrong:** Designing an abstract `Renderer` trait with pluggable backends (SVG/Cairo, GPU/wgpu, etc.) before having a single working pipeline. Manim itself went through this -- maintaining parallel Cairo and OpenGL hierarchies with a metaclass swap. The abstraction boundary is drawn in the wrong place because you don't yet know what renderers actually need.

**Why it happens:** "Good architecture" instincts say to abstract early. Manim's dual-backend complexity feels like a cautionary tale that calls for better abstraction. But the abstraction you design before building one complete pipeline will be wrong.

**Consequences:** The renderer trait either becomes too thin (useless) or too thick (leaky abstraction). Every new feature requires changes to the trait plus all implementations. Development velocity drops to a crawl because every change touches multiple layers.

**Prevention:**
- Commit to the SVG-to-rasterize-to-encode pipeline as the only pipeline for v1. This is already in PROJECT.md -- hold the line.
- Build the pipeline as concrete types, not traits. `SvgFrame`, `RasterizedFrame`, `VideoEncoder` -- no `dyn Renderer`.
- Extract a renderer trait only if/when you actually add a second backend (wgpu for real-time preview, for instance), and let the second backend's needs shape the abstraction.

**Detection:** If you're writing `trait Renderer` or `trait Backend` before you have a working video output, you're abstracting too early.

**Phase relevance:** All phases. Resist this throughout v1.

**Confidence:** HIGH -- Manim's own dual-hierarchy complexity is well-documented (DeepWiki analysis of ConvertToOpenGL metaclass). The YAGNI principle applies strongly here.

---

### Pitfall 4: SVG-per-Frame Performance Wall

**What goes wrong:** The pipeline (generate SVG string per frame, parse it with resvg/usvg, rasterize with tiny-skia, encode) works for prototyping but hits a wall at scale. Generating and parsing SVG XML for every frame at 30-60fps for minutes of video means tens of thousands of SVG documents. String allocation, XML parsing, and tree construction dominate CPU time.

**Why it happens:** SVG is a natural interchange format and resvg is excellent. The pipeline is conceptually clean. But SVG is a serialization format, not an in-memory scene representation. Round-tripping through text on every frame is wasteful.

**Consequences:** Rendering a 60-second video at 30fps (1800 frames) takes minutes instead of seconds. Users wait too long during iteration, killing the development workflow that makes Manim productive.

**Prevention:**
- Use SVG generation for the first working prototype only (Phase 1). Get something on screen fast.
- In Phase 2, move to direct tiny-skia drawing commands. Build scene objects that know how to paint themselves onto a `tiny_skia::Pixmap` directly, bypassing SVG entirely for the hot rendering loop.
- Keep SVG export as a secondary output format (for debugging, for static images), but not in the frame-by-frame rendering pipeline.
- Benchmark early: render 100 frames with SVG pipeline, measure time, extrapolate to a 2-minute video. This will motivate the transition.

**Detection:** Profile the rendering loop. If >50% of time is in SVG generation/parsing rather than actual rasterization, you've hit this wall.

**Phase relevance:** Phase 1 uses SVG (acceptable for prototyping). Phase 2 must migrate to direct rendering. Delaying past Phase 2 means the performance problem compounds as more features are added.

**Confidence:** MEDIUM -- based on architectural reasoning about the overhead of text serialization/parsing per frame. No specific benchmarks found for this exact pipeline, but the overhead of XML parsing per frame is well-understood.

---

## Moderate Pitfalls

### Pitfall 5: Trait Object Proliferation for Animatable Types

**What goes wrong:** Defining a `trait Animatable` and using `Box<dyn Animatable>` everywhere so that any scene element can be animated. This prevents the compiler from inlining interpolation code, introduces vtable overhead in the hot loop (interpolation runs per-frame per-object), and makes it impossible to store heterogeneous animatable properties without type erasure.

**Prevention:**
- Use an enum-based approach for the finite set of scene element types rather than open trait objects. `enum SceneElement { Axes(Axes), Curve(Curve), Label(Label), ... }` allows match-based dispatch which the compiler can optimize.
- Reserve trait objects for user-extensible plugin points (if ever needed), not for the core element types which are known at compile time.
- For the interpolation trait specifically, use generics: `fn interpolate<T: Lerp>(from: &T, to: &T, t: f64) -> T` rather than `fn interpolate(&self, other: &dyn Animatable, t: f64) -> Box<dyn Animatable>`.

**Detection:** If you have `Vec<Box<dyn Animatable>>` in your scene, reconsider.

**Phase relevance:** Phase 1 (core type design). Hard to change later once the rest of the codebase depends on trait objects.

**Confidence:** HIGH -- standard Rust performance guidance. Static dispatch vs dynamic dispatch tradeoffs are thoroughly documented.

---

### Pitfall 6: Interpolation Generality Trap

**What goes wrong:** Trying to make every property of every object animatable from the start. Position, color, opacity, stroke width, font size, curve control points, axis ranges, label text... Building a generic property animation system before understanding which transitions actually matter for the use case.

**Prevention:**
- Start with the transitions that GAM visualizations actually need: object fade-in/out, curve morphing (changing y-values along shared x-range), axis rescaling, and sequential element appearance (the "build" animation in presentations).
- Implement these as concrete animation types: `FadeIn`, `Transform`, `MorphCurve`. Each knows exactly what properties it interpolates.
- Generalize only after you've built 3-4 concrete animation types and see the actual shared patterns.

**Detection:** If you're designing a `PropertyAnimator<T>` generic system before you have one working animation, you're over-engineering.

**Phase relevance:** Phase 1-2. Build concrete animations first, extract patterns in Phase 3.

**Confidence:** MEDIUM -- based on Manim's own evolution (started with specific animation types like `Transform`, `FadeIn`, `Write`, then generalized).

---

### Pitfall 7: FFmpeg Integration Complexity

**What goes wrong:** Using low-level FFmpeg bindings (`ffmpeg-sys-next`, `rsmpeg`) that require linking against system FFmpeg libraries. Build failures on different platforms, version mismatches, complex build.rs scripts, and the need to understand FFmpeg's codec/muxer/format API.

**Prevention:**
- Use `ffmpeg-sidecar` which wraps the FFmpeg CLI binary via stdin/stdout pipes. It treats FFmpeg as an external process, avoiding all linking complexity. Users just need `ffmpeg` on PATH (which is ubiquitous).
- Pipe raw RGB/RGBA frames to ffmpeg's stdin, let ffmpeg handle encoding. This is exactly the pattern Manim uses (it shells out to ffmpeg).
- The performance overhead of piping vs. in-process encoding is negligible compared to the actual encoding work.
- Only consider native FFmpeg bindings if profiling shows the pipe is a bottleneck (it won't be for offline rendering).

**Detection:** If your build.rs is trying to find/link FFmpeg libraries, or if CI is failing on FFmpeg version mismatches, switch to the sidecar approach.

**Phase relevance:** Phase 1 (video output). Get this right early so the full pipeline works end-to-end.

**Confidence:** HIGH -- `ffmpeg-sidecar` exists specifically for this use case, and Manim's own approach of shelling out to ffmpeg validates the pattern.

---

### Pitfall 8: Coordinate System Confusion

**What goes wrong:** Mixing up coordinate systems between mathematical space (y-up, origin at center), SVG space (y-down, origin at top-left), and pixel space (y-down, integer coordinates). Objects render in wrong positions, axes are inverted, and transforms compose incorrectly.

**Prevention:**
- Define a single canonical "world" coordinate system early (mathematical: y-up, origin at center, floating-point units).
- All scene objects and user-facing APIs operate in world coordinates exclusively.
- The coordinate transform to rendering space (SVG/pixel) happens in exactly one place: the camera/viewport module. Never in object code.
- Write a `Camera` struct that handles the world-to-screen transform, including aspect ratio, zoom, and pan. All rendering goes through this transform.

**Detection:** If any scene element type contains pixel-coordinate logic, or if you're flipping y-coordinates in more than one location, the abstraction is leaking.

**Phase relevance:** Phase 1 (core math). Must be settled before rendering begins.

**Confidence:** HIGH -- universal computer graphics pitfall, well-documented in every graphics programming resource.

---

### Pitfall 9: Text and Label Rendering Rabbit Hole

**What goes wrong:** Attempting sophisticated text rendering (mathematical notation, subscripts, font selection, text-along-path) early. resvg supports text but has limitations: no embedded fonts in older versions, no textPath, and font loading has platform-specific rough edges. Trying to get "beautiful math labels" becomes a multi-week detour.

**Prevention:**
- Phase 1 labels are simple: axis tick values ("0", "5", "10") and plain-text titles. Use basic SVG `<text>` elements or direct tiny-skia text rendering with a single bundled font.
- Explicitly defer mathematical typesetting. PROJECT.md already marks LaTeX as out of scope -- extend this to all complex text layout.
- For GAM visualizations, labels are primarily numeric values and short axis names. This is achievable with basic font rendering.
- If richer text is needed later, consider pre-rendering text to PNG with an external tool and compositing it, rather than building a text layout engine.

**Detection:** If you're investigating font shaping libraries (rustybuzz, swash) or trying to render subscripts, you've fallen in.

**Phase relevance:** Phase 1 uses basic text only. Phase 3+ can revisit if needed.

**Confidence:** HIGH -- resvg's own documentation acknowledges text rendering limitations, and Manim's LaTeX dependency is its most common installation problem.

---

## Minor Pitfalls

### Pitfall 10: Scope Creep Toward General-Purpose Graphics Engine

**What goes wrong:** Starting to add features that serve "general animation" rather than the GAM visualization use case: 3D support, camera fly-throughs, particle effects, interactive elements. The project scope balloons from "animated GAM plots" to "Rust Manim clone" to "general motion graphics toolkit."

**Prevention:**
- Keep a strict scope document (PROJECT.md serves this role). Every feature request gets evaluated against: "Does this help render an animated GAM visualization for a presentation?"
- The GAM use case drives the API: axes, curves, confidence bands, sequential reveals. If a feature doesn't serve this, it goes to a "someday" list.
- Generalization happens by making the GAM-specific API composable, not by adding unrelated capabilities.

**Detection:** If you're implementing features you haven't needed in an actual visualization, stop.

**Phase relevance:** All phases. Review scope at every phase boundary.

**Confidence:** HIGH -- PROJECT.md already acknowledges this risk ("start as personal tooling, generalize as patterns emerge").

---

### Pitfall 11: Over-Engineering the Easing/Timing System

**What goes wrong:** Building a complex timeline system with keyframes, tracks, parallel/sequential composition, easing-per-property, and cubic bezier curves before having a single working animation. The timing system becomes its own mini-project.

**Prevention:**
- Phase 1 timing: each animation has a duration and a single easing function (use the `mina` crate or implement the 5-6 standard easings: linear, ease-in, ease-out, ease-in-out, smooth).
- Animations play sequentially (one after another) or simultaneously (grouped). That's it for v1.
- No keyframe tracks, no timeline scrubbing, no per-property easing. These are real-time editor features, not offline rendering necessities.

**Detection:** If you're designing a `Timeline` struct with `Track` children before your first `FadeIn` animation works, simplify.

**Phase relevance:** Phase 2 (animation system). Keep it simple until real use cases demand complexity.

**Confidence:** MEDIUM -- based on analysis of Manim's own evolution from simple play()/wait() to more complex timing.

---

### Pitfall 12: Not Testing Visual Output Early Enough

**What goes wrong:** Building layers of abstraction (scene graph, animation system, interpolation) without actually rendering frames and watching the output. Bugs in coordinate transforms, color handling, or animation timing are invisible until you see the pixels.

**Prevention:**
- Get a single static frame rendering end-to-end in Phase 1, week 1. Even if it's just a colored rectangle.
- Keep a set of visual regression test cases: render known scenes to PNG, compare against reference images (pixel diff or perceptual hash).
- Use `insta` crate for snapshot testing of SVG output (text-based snapshots of generated SVGs).

**Detection:** If you've built the scene graph and animation system but haven't seen a rendered frame yet, you're flying blind.

**Phase relevance:** Phase 1, from the very start.

**Confidence:** HIGH -- universal software development principle, amplified by the visual nature of this project.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Core data model (Phase 1) | Scene graph ownership (#1) | Use slotmap/arena from day one |
| API design (Phase 1) | Macro DSL trap (#2) | Builder pattern first, macro later |
| Rendering pipeline (Phase 1) | Premature abstraction (#3) | Concrete SVG pipeline, no traits |
| Video output (Phase 1) | FFmpeg linking hell (#7) | Use ffmpeg-sidecar (CLI wrapper) |
| Coordinate math (Phase 1) | Coordinate confusion (#8) | Single canonical world space + Camera |
| Text/labels (Phase 1-2) | Text rendering rabbit hole (#9) | Basic text only, defer rich layout |
| Animation system (Phase 2) | Interpolation generality trap (#6) | Concrete animation types first |
| Performance (Phase 2) | SVG-per-frame wall (#4) | Migrate to direct tiny-skia drawing |
| Type system (Phase 1-2) | Trait object proliferation (#5) | Enum dispatch for known types |
| Scope management (all) | Scope creep (#10) | GAM use case gates every feature |
| Animation timing (Phase 2) | Over-engineered timeline (#11) | Sequential + parallel, single easing |
| Quality assurance (all) | No visual testing (#12) | Render frames from day one |

## Sources

- [Arenas in Rust - Manish Goregaokar](https://manishearth.github.io/blog/2021/03/15/arenas-in-rust/)
- [Graphs and arena allocation - Rust for C++ Programmers](https://aminb.gitbooks.io/rust-for-c/content/graphs/index.html)
- [generational-arena (handles ABA problem)](https://github.com/fitzgen/generational-arena)
- [slotmap crate documentation](https://docs.rs/slotmap/latest/slotmap/index.html)
- [scene-graph crate (thunderdome-backed)](https://docs.rs/scene-graph)
- [Rust static vs dynamic dispatch](https://softwaremill.com/rust-static-vs-dynamic-dispatch/)
- [Builder pattern in Rust - LogRocket](https://blog.logrocket.com/build-rust-api-builder-pattern/)
- [Manim architecture - DeepWiki](https://deepwiki.com/ManimCommunity/manim)
- [ffmpeg-sidecar crate](https://lib.rs/crates/ffmpeg-sidecar)
- [resvg - SVG rendering library](https://github.com/linebender/resvg)
- [tiny-skia - 2D rendering](https://github.com/linebender/tiny-skia)
- [mina crate - easing/interpolation](https://docs.rs/mina)
- [Manim Community documentation](https://docs.manim.community/en/stable/)
