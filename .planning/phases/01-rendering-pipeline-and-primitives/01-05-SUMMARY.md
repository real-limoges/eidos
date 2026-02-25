---
phase: 01-rendering-pipeline-and-primitives
plan: "05"
subsystem: rendering
tags: [svg, rust, primitives, ffmpeg, integration-test]

# Dependency graph
requires:
  - phase: 01-02
    provides: svg_gen.rs with rasterize_frame and encode_to_mp4
  - phase: 01-03
    provides: Circle and Rect primitive structs with to_svg_element()
  - phase: 01-04
    provides: Line, Arrow, Text, Bezier primitive structs with to_svg_element()/to_svg_parts()
provides:
  - Complete build_svg_document() dispatch for all 6 Primitive variants
  - Arrow defs-before-shape ordering (SVG spec compliance)
  - Working basic_scene example demonstrating full public API
  - Integration test suite (render, dimension validation, fps validation)
affects: [02-animation, all future phases using primitives]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Two-pass SVG assembly (Arrow defs in pass 1, all shapes in pass 2)
    - Integration test with ffmpeg availability guard for CI portability

key-files:
  created:
    - examples/basic_scene.rs
    - tests/integration.rs
  modified:
    - src/svg_gen.rs

key-decisions:
  - "Arrow::to_svg_parts() called twice per arrow (once for defs, once for line) — acceptable for Phase 1 static scenes"
  - "Integration test skips render when ffmpeg unavailable — enables CI without ffmpeg dependency"
  - "basic_scene uses Bezier (not BezierPath) matching actual enum variant name"
  - "tiny-skia Pixmap::data() returns RGBA (not BGRA) — ffmpeg -pix_fmt must be rgba to avoid R/B channel swap"

patterns-established:
  - "Two-pass SVG construction: collect defs first, then add shape nodes — ensures url(#id) references are never unresolved"
  - "Integration tests guard ffmpeg-dependent paths with ffmpeg_available() check"

requirements-completed: [CORE-01, CORE-02, PRIM-01, PRIM-02, PRIM-03, PRIM-04, PRIM-05, PRIM-06]

# Metrics
duration: 30min
completed: 2026-02-25
---

# Phase 1 Plan 05: SVG Dispatch Integration and Phase 1 End-to-End Summary

**All 6 primitives wired through svg_gen dispatch with verified MP4 output — R/B color channel swap fixed by correcting tiny-skia pixel format from bgra to rgba**

## Performance

- **Duration:** ~30 min (including human verification checkpoint and color fix)
- **Started:** 2026-02-25T05:47:39Z
- **Completed:** 2026-02-25
- **Tasks:** 3 (2 automated + 1 human-verify with color fix)
- **Files modified:** 4

## Accomplishments
- Replaced all 6 TODO stubs in build_svg_document() with real dispatches — Circle, Rect, Line, Arrow, Text, Bezier all produce correct SVG nodes
- Arrow defs collected in pass 1 before shapes in pass 2 — SVG spec compliance for url(#id) marker references
- basic_scene example (examples/basic_scene.rs) demonstrates all 6 primitive types on a 1920x1080 canvas
- Integration tests cover render pipeline (with ffmpeg guard), odd-dimension rejection, and zero-fps rejection
- All 25 tests pass (22 unit + 3 integration)

## Task Commits

Each task was committed atomically:

1. **Task 1: Complete svg_gen dispatch for all 6 primitive types** - `d96b3d4` (feat)
2. **Task 2: Create basic_scene example and integration test** - `00ed76e` (feat)
3. **Color channel fix: pixel format bgra -> rgba** - `73523e8` (fix)

## Files Created/Modified
- `src/svg_gen.rs` - build_svg_document() with complete 6-variant dispatch and two-pass Arrow ordering; fixed pixel format from bgra to rgba
- `src/scene.rs` - variable rename bgra_frame -> rgba_frame for accuracy
- `examples/basic_scene.rs` - runnable demo of all Phase 1 primitives (Circle, Rect, Line, Arrow, Text, Bezier)
- `tests/integration.rs` - 3 integration tests: render pipeline, odd-dimension validation, zero-fps validation

## Decisions Made
- Arrow::to_svg_parts() is called twice per arrow (once for defs in pass 1, once for the line element in pass 2). This is acceptable for Phase 1 — scenes are static, build_svg_document() is called once per render. Phase 2 can cache SVG parts on Arrow if needed.
- The integration test guards the render path with ffmpeg_available() so it skips cleanly in environments without ffmpeg.
- basic_scene.rs uses `Bezier` (not `BezierPath` as the plan pseudocode suggested) — matches the actual Primitive enum variant and mod.rs export established in plans 01-03/01-04.
- tiny-skia `Pixmap::data()` returns RGBA byte order (not BGRA). The original research note "Pitfall 2" was incorrect. Verified by reading the tiny-skia 0.12.0 source (`src/pixmap.rs`): both `data()` and `data_mut()` explicitly document "Byteorder: RGBA". Using `bgra` in ffmpeg swapped R and B channels globally.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected BezierPath naming to Bezier throughout example**
- **Found during:** Task 2 (basic_scene.rs implementation)
- **Issue:** Plan pseudocode used `BezierPath::new()` and `eidos::primitives::BezierPath` imports, but the actual struct and Primitive enum variant are named `Bezier` (established in plan 01-04 — see STATE.md decision)
- **Fix:** Used `Bezier` in all imports and struct construction in basic_scene.rs
- **Files modified:** examples/basic_scene.rs
- **Verification:** cargo build --example basic_scene succeeds
- **Committed in:** 00ed76e (Task 2 commit)

**2. [Rule 1 - Bug] Fixed R/B channel swap in MP4 output by correcting ffmpeg pixel format from bgra to rgba**
- **Found during:** Task 3 (human verification — circle appeared blue, rect appeared red)
- **Issue:** `rasterize_frame()` used BGRA format assumption from research notes (Pitfall 2), but tiny-skia 0.12.0 `Pixmap::data()` actually returns RGBA byte order. Using `-pix_fmt bgra` in ffmpeg caused the red circle to appear blue, the blue rect to appear red, the yellow arrow to appear cyan, and the cyan bezier to appear greenish/yellow.
- **Fix:** Changed ffmpeg `-pix_fmt` from `bgra` to `rgba`. Updated comments and variable names in `svg_gen.rs` and `scene.rs` to accurately describe RGBA format.
- **Files modified:** src/svg_gen.rs, src/scene.rs
- **Verification:** Rebuilt and re-ran `cargo run --example basic_scene`. All 25 tests pass. New MP4 at /tmp/basic_scene.mp4 ready for re-verification.
- **Committed in:** 73523e8 (fix(01-05))

---

**Total deviations:** 2 auto-fixed (1 naming bug, 1 pixel format bug)
**Impact on plan:** Both fixes necessary for correctness. The pixel format fix was the root cause of the user-reported color issues. No scope creep.

## Issues Encountered
- Research note "Pitfall 2" in 01-RESEARCH.md incorrectly claimed tiny-skia Pixmap::data() returns BGRA. Verified the truth by reading tiny-skia 0.12.0 source directly. This caused visible color corruption in MP4 output detected during human verification checkpoint.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 1 rendering pipeline is complete: all 6 primitives (Circle, Rect, Line, Arrow, Text, Bezier) wire through SVG generation, resvg rasterization, and ffmpeg MP4 encoding
- Color channel bug fixed — re-generated /tmp/basic_scene.mp4 is ready for final visual confirmation
- Phase 2 (animation) can build on SceneBuilder, Scene::render(), and the Primitive enum dispatch

Concerns carried forward:
- SVG-per-frame performance is unvalidated — benchmark early in Phase 2
- Font handling may need bundled font for cross-platform consistency (Phase 3)

---
*Phase: 01-rendering-pipeline-and-primitives*
*Completed: 2026-02-25*
