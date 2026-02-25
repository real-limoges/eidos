---
phase: 01-rendering-pipeline-and-primitives
plan: "02"
subsystem: core
tags: [rust, svg, resvg, tiny-skia, ffmpeg, fontdb, ttf-noto-sans, rendering, pipeline]

# Dependency graph
requires:
  - phase: 01-01
    provides: EidosError, Color, module skeleton, Cargo dependencies (svg, resvg, ttf-noto-sans)
provides:
  - Scene struct with new(), duration(), render() — validated video config and closure API
  - SceneBuilder with add() — accumulates primitives during render closure
  - Primitive enum in primitives/mod.rs with 6 variants (Circle, Rect, Line, Arrow, Text, Bezier)
  - build_svg_document() — SVG Document assembly with background rect and stub primitive dispatch
  - rasterize_frame() — resvg + tiny-skia BGRA8 rasterization using Arc<fontdb>
  - encode_to_mp4() — ffmpeg subprocess pipeline with stdin piping and proper EOF handling
affects:
  - 01-03-primitive-shapes
  - 01-04-advanced-primitives
  - 01-05-render-pipeline

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Scene-with-fontdb-arc pattern (fontdb initialized once at Scene::new(), stored as Arc<fontdb::Database>, passed to rasterize_frame — avoids per-frame font loading)
    - Builder-closure render API (scene.render(|b| { b.add(primitive); }, path) — Phase 2 contract established here)
    - Two-pass SVG assembly (Definitions first for Arrow markers, then shape nodes — prevents SVG spec violation)
    - Subprocess-stdin-drop pattern (drop stdin before child.wait() to send EOF to ffmpeg — prevents blocking)

key-files:
  created: []
  modified:
    - src/scene.rs
    - src/svg_gen.rs
    - src/primitives/mod.rs
    - src/primitives/circle.rs
    - src/primitives/rect.rs
    - src/primitives/line.rs
    - src/primitives/arrow.rs
    - src/primitives/text.rs
    - src/primitives/bezier.rs

key-decisions:
  - "fontdb stored as Arc<fontdb::Database> not fontdb::Database — resvg 0.47 Options.fontdb field requires Arc, discovered from actual API vs plan spec"
  - "Primitive enum with stub struct variants added to primitives/mod.rs in this plan — required for scene.rs to compile ahead of plans 01-03/01-04"
  - "usvg::Tree::from_str() in resvg 0.47 takes 2 args (text, options) not 3 — fontdb goes in Options.fontdb field, auto-fixed from plan spec"
  - "stdin dropped via block scope borrow drop + explicit take() — belt-and-suspenders ensures ffmpeg receives EOF before wait()"

patterns-established:
  - "fontdb pattern: Arc<fontdb::Database> initialized in Scene::new(), cloned into Options.fontdb per rasterize_frame call — one allocation, many readers"
  - "render closure pattern: Fn(&mut SceneBuilder) — closure runs once per frame in Phase 1 (static), will run per animation step in Phase 2"
  - "SVG assembly pattern: two-pass (defs then shapes), matching document dimensions to viewBox for 1:1 pixel mapping"

requirements-completed: [CORE-01, CORE-02]

# Metrics
duration: 2min
completed: 2026-02-25
---

# Phase 1 Plan 02: Scene struct and full SVG-to-MP4 rendering pipeline via resvg and ffmpeg Summary

**Scene::new() with eager config validation, render() closure API, resvg rasterization via fontdb Arc, and ffmpeg subprocess pipeline producing H.264 MP4 from static SVG frames**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-25T05:32:22Z
- **Completed:** 2026-02-25T05:35:21Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Scene::new() validates even dimensions (H.264 requirement), non-zero fps, and ffmpeg presence at construction — returns EidosError early with clear messages
- fontdb initialized once with Noto Sans via Arc<fontdb::Database>, reused per frame — no per-frame font loading
- render() closure API (Fn(&mut SceneBuilder)) established as the Phase 2 animation contract — SceneBuilder::add() accumulates primitives
- build_svg_document() assembles SVG with proper width/height/viewBox parity and two-pass def/shape ordering
- rasterize_frame() converts SVG string to BGRA8 bytes via resvg tiny-skia pixmap
- encode_to_mp4() spawns ffmpeg with -pix_fmt bgra, pipes frames to stdin, drops stdin before wait() for correct EOF signaling
- Primitive enum with all 6 stub variants added so downstream plans 01-03/01-04 can implement without restructuring

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement Scene struct with config validation and render closure API** - `cba0b5d` (feat)
2. **Task 2: Implement SVG document assembly and rasterization pipeline** - `15527cd` (feat)

**Plan metadata:** (docs commit below)

## Files Created/Modified

- `src/scene.rs` - Scene struct with new(), duration(), render(); SceneBuilder with add(); full validation and pipeline orchestration
- `src/svg_gen.rs` - build_svg_document() (SVG assembly), rasterize_frame() (resvg rasterization), encode_to_mp4() (ffmpeg subprocess)
- `src/primitives/mod.rs` - Added Primitive enum with Circle, Rect, Line, Arrow, Text, Bezier variants
- `src/primitives/circle.rs` - Added Circle stub struct (full implementation in plan 01-03)
- `src/primitives/rect.rs` - Added Rect stub struct (full implementation in plan 01-03)
- `src/primitives/line.rs` - Added Line stub struct (full implementation in plan 01-03)
- `src/primitives/arrow.rs` - Added Arrow stub struct (full implementation in plan 01-04)
- `src/primitives/text.rs` - Added Text stub struct (full implementation in plan 01-04)
- `src/primitives/bezier.rs` - Added Bezier stub struct (full implementation in plan 01-04)

## Decisions Made

- fontdb stored as Arc<fontdb::Database> rather than fontdb::Database — resvg 0.47 requires Arc for Options.fontdb; also enables cheap clone for multi-frame use in Phase 2
- Primitive enum added to primitives/mod.rs in this plan (not 01-03) — needed for SceneBuilder and svg_gen dispatch to compile; stub variants reference empty structs forward-declared in each module
- stdin EOF sent via block scope drop + explicit take() — the block scope drop terminates the mutable borrow, then take() ensures the Option is cleared before wait()

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed usvg::Tree::from_str() 3-argument call — resvg 0.47 API is 2-argument**
- **Found during:** Task 2 (rasterize_frame implementation)
- **Issue:** Plan specified `usvg::Tree::from_str(svg_str, &options, fontdb)` but resvg 0.47 only takes 2 arguments — fontdb is passed via `Options.fontdb` field (Arc<fontdb::Database>), not a third parameter
- **Fix:** Changed to `options.fontdb = fontdb.clone()` then `usvg::Tree::from_str(svg_str, &options)`. Updated Scene.fontdb field to Arc<fontdb::Database> and rasterize_frame() signature accordingly.
- **Files modified:** src/svg_gen.rs, src/scene.rs
- **Verification:** cargo build succeeds with zero errors
- **Committed in:** cba0b5d / 15527cd (included in task commits)

---

**Total deviations:** 1 auto-fixed (Rule 1 - API signature mismatch)
**Impact on plan:** Required fix for compilation. No scope change, same architectural intent — fontdb is still initialized once in Scene::new() and reused per frame.

## Issues Encountered

- resvg 0.47 fontdb API differs from plan spec: `from_str` takes (text, options) not (text, options, fontdb). The fontdb is set on `Options.fontdb: Arc<fontdb::Database>`. Discovered on first cargo build, fixed immediately.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Rendering pipeline complete: Scene::new() → render() closure → build_svg_document() → rasterize_frame() → encode_to_mp4()
- Primitive enum with all 6 stubs in place — plans 01-03 and 01-04 can replace stub structs with real implementations immediately
- TODO(01-05) markers in svg_gen.rs build_svg_document() match arms — plan 01-05 completes the dispatch
- Phase 2 animation contract established: render() closure Fn(&mut SceneBuilder) will be called per-frame with animated state

## Self-Check: PASSED

- src/scene.rs: FOUND
- src/svg_gen.rs: FOUND
- src/primitives/mod.rs: FOUND
- 01-02-SUMMARY.md: FOUND
- Commit cba0b5d: FOUND
- Commit 15527cd: FOUND

---
*Phase: 01-rendering-pipeline-and-primitives*
*Completed: 2026-02-25*
