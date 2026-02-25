# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-24)

**Core value:** A Rust-native way to produce beautiful, animated data visualizations with a declarative API -- no Python, no GUI, just code that describes a scene and produces a video.
**Current focus:** Phase 1: Rendering Pipeline and Primitives

## Current Position

Phase: 1 of 4 (Rendering Pipeline and Primitives)
Plan: 5 of 5 in current phase (plan 01-05 complete — Phase 1 DONE)
Status: Phase 1 complete
Last activity: 2026-02-25 -- Plan 01-05 color fix applied (bgra->rgba), all 25 tests pass, /tmp/basic_scene.mp4 re-generated

Progress: [█████░░░░░] 25% (5/20 plans est.)

## Performance Metrics

**Velocity:**
- Total plans completed: 4
- Average duration: 2 min
- Total execution time: 0.12 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-rendering-pipeline-and-primitives | 5 | 11 min | 2 min |

**Recent Trend:**
- Last 5 plans: 01-01 (2 min), 01-02 (2 min), 01-03 (2 min), 01-04 (3 min), 01-05 (2 min)
- Trend: Consistent 2-3 min/plan

*Updated after each plan completion*
| Phase 01-rendering-pipeline-and-primitives P05 | 30 | 3 tasks | 4 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: 4-phase structure derived from requirement clusters (pipeline/primitives, animation, data viz, GAM viz)
- [Roadmap]: Phase 1 combines pipeline and primitives (vertical slice, not horizontal layers)
- [Research]: Arena-based scene graph (slotmap), enum dispatch over trait objects, builder API before macros
- [01-01]: EidosError uses two variants (InvalidConfig, RenderFailed) — opaque to callers, covers both distinct failure modes
- [01-01]: Color uses u8 RGB components — sufficient for SVG display, avoids f32 ergonomics issues
- [01-01]: Module skeleton created upfront so all Wave 2 plans compile without restructuring lib.rs
- [01-01]: slotmap deferred to Phase 2 — not needed in Phase 1
- [Phase 01-02]: fontdb stored as Arc<fontdb::Database> — resvg 0.47 Options.fontdb requires Arc, also enables cheap clone for Phase 2 multi-frame use
- [Phase 01-02]: Primitive enum added in plan 01-02 (not 01-03) — needed for SceneBuilder and svg_gen dispatch to compile ahead of primitive struct implementations
- [Phase 01-rendering-pipeline-and-primitives]: fill() returns Self (no validation); stroke()/opacity() return Result<Self, EidosError> (eager validation at call site)
- [Phase 01-rendering-pipeline-and-primitives]: to_svg_element() returns concrete SVG node type (not boxed trait) for efficiency
- [Phase 01-rendering-pipeline-and-primitives]: TSpan::new(content) in svg 0.18 takes content as constructor arg — no .add(TextNode::new()) needed
- [Phase 01-rendering-pipeline-and-primitives]: Arrow uses AtomicU64 counter for unique SVG marker IDs — zero external dependencies, thread-safe
- [Phase 01-rendering-pipeline-and-primitives]: Bezier named to match mod.rs export and Primitive enum variant (not BezierPath as in plan pseudocode)
- [Phase 01-05]: Arrow::to_svg_parts() called twice per arrow — acceptable for Phase 1 static scenes, cache in Phase 2 if needed
- [Phase 01-05]: Integration test guards render path with ffmpeg_available() — CI portability without hard ffmpeg dependency
- [Phase 01-rendering-pipeline-and-primitives]: tiny-skia Pixmap::data() returns RGBA (not BGRA) — ffmpeg -pix_fmt must be rgba to avoid R/B channel swap

### Pending Todos

None yet.

### Blockers/Concerns

- [Research]: SVG-per-frame performance is unvalidated -- benchmark early in Phase 1
- [Research]: Font handling in resvg may need bundled font for cross-platform consistency (Phase 3)

## Session Continuity

Last session: 2026-02-25
Stopped at: Completed 01-05-PLAN.md fully — color channel fix (bgra->rgba) applied, all tasks done, SUMMARY.md updated, Phase 1 complete
Resume file: None
