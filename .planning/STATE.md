# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-24)

**Core value:** A Rust-native way to produce beautiful, animated data visualizations with a declarative API -- no Python, no GUI, just code that describes a scene and produces a video.
**Current focus:** Phase 2.5: Tech Debt Cleanup

## Current Position

Phase: 2.5 of 4 (Tech Debt Cleanup)
Plan: 1 of 1 in current phase (plan 02.5-01 complete)
Status: Active
Last activity: 2026-02-25 -- Plan 02.5-01 complete: encode_to_mp4 deprecated, LineState+TextState coverage added to example and integration tests

Progress: [████████░░] 45% (9/20 plans est.)

## Performance Metrics

**Velocity:**
- Total plans completed: 6
- Average duration: 2 min
- Total execution time: 0.17 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-rendering-pipeline-and-primitives | 5 | 11 min | 2 min |
| 02-animation-engine | 1 | 5 min | 5 min |
| 02.5-tech-debt-cleanup | 1 | 2 min | 2 min |

**Recent Trend:**
- Last 5 plans: 01-04 (3 min), 01-05 (2 min), 02-01 (5 min), 02-02 (2 min), 02-03 (5 min), 02.5-01 (2 min)
- Trend: Consistent 2-5 min/plan

*Updated after each plan completion*
| Phase 01-rendering-pipeline-and-primitives P05 | 30 | 3 tasks | 4 files |
| Phase 02-animation-engine P01 | 5 | 1 commit | 10 files |
| Phase 02-animation-engine P02 | 2 | 2 tasks | 2 files |
| Phase 02-animation-engine P03 | 5 | 3 tasks | 4 files |
| Phase 02.5-tech-debt-cleanup P01 | 2 | 2 tasks | 3 files |

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
- [Phase 02-01]: EaseInOut at exactly t=0.5 returns same value as Linear (symmetric function property — not a bug); test uses t=0.25 quarter-point
- [Phase 02-01]: LineState includes opacity field (Line struct has opacity; plan pseudocode omitted it)
- [Phase 02-01]: Color channels in State structs are f64 (0.0..=255.0) — clamped+cast to u8 only at to_*() time; no arithmetic overflow
- [Phase 02-01]: keyframe_derive #[derive(CanTween)] works cleanly on all four State structs (f64-only fields required)
- [Phase 02-animation-engine]: encode_to_mp4_animated takes frame index (u64) not scene time; Scene::render() computes t_secs so svg_gen stays fps-agnostic
- [Phase 02-animation-engine]: render_static() delegates to render() with |s, _t| wrapper — Phase 1 callers updated in 02-03, not here
- [Phase 02-animation-engine]: No new decisions — parallel Tween composition wired together without architectural choices
- [Phase 02.5-01]: #[allow(dead_code)] paired with #[deprecated] on encode_to_mp4 — suppresses unused warning while communicating deprecation; removal deferred to Phase 3
- [Phase 02.5-01]: LineState and TextState use submodule import paths (eidos::primitives::line::LineState) — primitives/mod.rs does not re-export State types

### Pending Todos

None yet.

### Blockers/Concerns

- [Research]: SVG-per-frame performance is unvalidated -- benchmark early in Phase 1
- [Research]: Font handling in resvg may need bundled font for cross-platform consistency (Phase 3)

## Session Continuity

Last session: 2026-02-25
Stopped at: Completed 02.5-01-PLAN.md — encode_to_mp4 deprecated, LineState+TextState coverage added to example and integration tests
Resume file: None
