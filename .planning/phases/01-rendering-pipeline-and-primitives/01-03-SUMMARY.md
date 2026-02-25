---
phase: 01-rendering-pipeline-and-primitives
plan: "03"
subsystem: primitives
tags: [rust, svg, builder-pattern, primitives, circle, rect]

# Dependency graph
requires:
  - phase: 01-rendering-pipeline-and-primitives
    plan: "01"
    provides: "Color, EidosError, module skeleton (primitives stubs)"
  - phase: 01-rendering-pipeline-and-primitives
    plan: "02"
    provides: "Primitive enum stub, SceneBuilder::add(), svg_gen dispatch stubs"
provides:
  - "Circle builder struct with new/fill/stroke/opacity/to_svg_element"
  - "Rect builder struct with new/fill/stroke/opacity/to_svg_element"
  - "From<Circle> and From<Rect> for Primitive"
  - "From impls for all 6 primitive types in mod.rs"
  - "6 unit tests validating eager validation on stroke/opacity"
affects:
  - "01-04-PLAN.md (Arrow, Text, Bezier builders follow same pattern)"
  - "01-05-PLAN.md (svg_gen dispatch uses to_svg_element())"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Builder pattern with eager validation: infallible methods return Self, methods that validate return Result<Self, EidosError>"
    - "to_svg_element() method convention for SVG node conversion"
    - "fill() -> Self, stroke() -> Result<Self, EidosError>, opacity() -> Result<Self, EidosError>"

key-files:
  created: []
  modified:
    - "src/primitives/mod.rs"
    - "src/primitives/circle.rs"
    - "src/primitives/rect.rs"

key-decisions:
  - "fill() returns Self (no validation needed — any Color is valid)"
  - "stroke() and opacity() return Result<Self, EidosError> (eager validation at call site)"
  - "Kept Bezier (not BezierPath) to match existing stub and svg_gen match arms from plan 01-02"
  - "to_svg_element() returns concrete SVG node type (not boxed trait) for efficiency"

patterns-established:
  - "Primitive builder pattern: new() is infallible; methods that can fail return Result<Self, EidosError>"
  - "SVG conversion: to_svg_element() on each primitive struct returns the concrete svg::node::element type"
  - "Tests co-located in module: #[cfg(test)] mod tests at bottom of each primitive file"

requirements-completed:
  - PRIM-01
  - PRIM-02

# Metrics
duration: 2min
completed: 2026-02-25
---

# Phase 1 Plan 03: Circle and Rect Primitive Builders Summary

**Circle and Rect builder structs with fill/stroke/opacity builder API, eager Result-returning validation, SVG node conversion, and 6 unit tests establishing the primitive pattern.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-24T21:38:16Z
- **Completed:** 2026-02-25T05:39:26Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Implemented Circle builder: new(cx, cy, r) infallible constructor, fill() -> Self, stroke() -> Result, opacity() -> Result, to_svg_element() -> svg::node::element::Circle
- Implemented Rect builder: new(x, y, width, height) infallible constructor, same fill/stroke/opacity/to_svg_element API
- Updated mod.rs with From<T> for Primitive implementations for all 6 primitive types plus pub re-exports
- All 6 unit tests pass: negative stroke width returns Err, out-of-range opacity returns Err, valid builder chain produces correct struct

## Task Commits

Each task was committed atomically:

1. **Task 1: Define Primitive enum and implement Circle builder** - `3743c3a` (feat)
2. **Task 2: Implement Rect builder** - `40079e5` (feat)

**Plan metadata:** (docs commit — pending)

## Files Created/Modified
- `src/primitives/mod.rs` - Updated with From<T> impls for all 6 primitive types and pub re-exports
- `src/primitives/circle.rs` - Full Circle builder replacing stub: new/fill/stroke/opacity/to_svg_element + 3 tests
- `src/primitives/rect.rs` - Full Rect builder replacing stub: new/fill/stroke/opacity/to_svg_element + 3 tests

## Decisions Made
- fill() returns Self because any Color value is valid — no runtime check needed
- stroke() and opacity() return Result<Self, EidosError> to give callers a clear, immediate error at the call site (eager validation pattern)
- Kept Bezier struct/variant name (not BezierPath) to stay consistent with the stub and svg_gen match arms already established in plan 01-02
- to_svg_element() returns the concrete SVG element type rather than a boxed trait to avoid unnecessary heap allocation

## Deviations from Plan

None - plan executed exactly as written. The only notable divergence from the plan's example code was using `Bezier` instead of `BezierPath` in mod.rs to maintain consistency with the existing codebase (this was already the correct state established in plan 01-02, not a new deviation).

## Issues Encountered

The `cargo test` command fails due to a missing `examples/basic_scene.rs` file referenced in Cargo.toml — this is a pre-existing issue from plan 01-02's Cargo.toml setup. All library tests pass via `cargo test --lib`. This is tracked as a deferred item to be resolved when the example is implemented.

## Next Phase Readiness
- Circle and Rect builders complete and tested — plan 01-04 can implement Arrow, Text, and Bezier following the same pattern
- The primitive builder pattern is established: infallible new(), Self-returning fill(), Result-returning stroke()/opacity()
- Plan 01-05 can use to_svg_element() on Circle and Rect directly when implementing svg_gen dispatch

---
*Phase: 01-rendering-pipeline-and-primitives*
*Completed: 2026-02-25*

## Self-Check: PASSED

- FOUND: src/primitives/circle.rs
- FOUND: src/primitives/rect.rs
- FOUND: src/primitives/mod.rs
- FOUND: 01-03-SUMMARY.md
- FOUND commit: 3743c3a (Task 1 - Primitive enum and Circle builder)
- FOUND commit: 40079e5 (Task 2 - Rect builder)
