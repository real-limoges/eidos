---
phase: 01-rendering-pipeline-and-primitives
plan: "04"
subsystem: primitives
tags: [rust, svg, line, arrow, text, bezier, atomic-counter, tspan, marker]

# Dependency graph
requires:
  - phase: 01-01
    provides: "EidosError, Color, module skeleton with stub primitives"
  - phase: 01-02
    provides: "Primitive enum with Line/Arrow/Text/Bezier variants for svg_gen dispatch"
provides:
  - "Line builder: new(x1,y1,x2,y2) + stroke_color/stroke_width/opacity + to_svg_element()"
  - "Arrow builder: new(x1,y1,x2,y2) + to_svg_parts() returning (Definitions, SvgLine) with unique atomic marker ID"
  - "Text builder: new(x,y,content) + fill/stroke/opacity/font_size/alignment/line_height + to_svg_element() with tspan multi-line"
  - "Bezier builder: new() + move_to/line_to/cubic_to/close (all Self) + stroke/fill/opacity + to_svg_element()"
affects:
  - "01-05 svg_gen dispatch — all four builders must be dispatched via Primitive enum match arms"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AtomicU64 counter for unique SVG IDs without UUID dependency"
    - "TSpan::new(content) API pattern for svg 0.18 crate (content passed to constructor, not .add())"
    - "Text::new('') empty parent element with tspan children for multi-line SVG text"
    - "Arrow returns (Definitions, SvgLine) tuple — defs must precede line in SVG document order"

key-files:
  created:
    - src/primitives/line.rs
    - src/primitives/arrow.rs
    - src/primitives/text.rs
    - src/primitives/bezier.rs
  modified: []

key-decisions:
  - "TSpan::new(content) in svg 0.18 takes content as constructor arg — no .add(TextNode::new()) needed"
  - "Text::new('') used for parent <text> element to hold tspan children — empty string avoids duplicate text content"
  - "Bezier named to match mod.rs export and Primitive enum (not BezierPath as in plan pseudocode)"
  - "Manual Debug/Clone implemented for PathCommand since derive is not available on enums with non-Copy fields (all f64 but needed explicit impl for clarity)"

patterns-established:
  - "Builder pattern: infallible setters return Self; setters with validation return Result<Self, EidosError>"
  - "Arrow uniqueness via global AtomicU64 counter — zero external dependencies, thread-safe"
  - "Multi-line text: split on \\n, first tspan dy='0', subsequent tspans dy='{line_height}em', each resets x"

requirements-completed: [PRIM-03, PRIM-04, PRIM-05, PRIM-06]

# Metrics
duration: 3min
completed: 2026-02-25
---

# Phase 1 Plan 04: Line, Arrow, Text, and Bezier Primitive Builders Summary

**Four SVG primitive builders completing the full Phase 1 set: Line/Arrow with stroke builders, Arrow with AtomicU64 unique marker IDs and (Definitions, SvgLine) return, Text with tspan multi-line support, and Bezier with cubic bezier path commands.**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-02-25T05:41:50Z
- **Completed:** 2026-02-25T05:44:41Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Line builder with stroke_color/stroke_width/opacity; to_svg_element() returns concrete SvgLine
- Arrow builder with AtomicU64 counter for unique per-instance SVG marker IDs; to_svg_parts() returns (Definitions, SvgLine) tuple ready for svg_gen two-pass insertion
- Text builder with full configurability (font_size, Alignment enum, line_height); to_svg_element() generates tspan children for multi-line content by splitting on `\n`
- Bezier path builder with move_to/line_to/cubic_to/close (all return Self); stroke() validates and returns Result; to_svg_element() assembles SVG path Data

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement Line and Arrow builders** - `79c722e` (feat)
2. **Task 2: Implement Text and Bezier builders** - `b0377e0` (feat)

## Files Created/Modified

- `src/primitives/line.rs` - Line builder with stroke_color/stroke_width/opacity/to_svg_element
- `src/primitives/arrow.rs` - Arrow builder with AtomicU64 unique marker IDs and to_svg_parts() returning (Definitions, SvgLine)
- `src/primitives/text.rs` - Text builder with Alignment enum, multi-line tspan support, font_size/line_height
- `src/primitives/bezier.rs` - Bezier path builder with PathCommand enum, move_to/line_to/cubic_to/close, to_svg_element()

## Decisions Made

- TSpan::new(content) in svg 0.18 takes content as constructor argument — the plan's approach of `.add(TextNode::new(...))` would not work. Using TSpan::new(line) with x/dy set attributes instead.
- Text::new("") used for parent `<text>` element — empty string avoids duplicate text content since all content lives in tspan children.
- Struct named `Bezier` (not `BezierPath`) to match the existing mod.rs pub use and Primitive enum variant from plan 01-02.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed TSpan::new() and Text::new() API calls for svg 0.18**
- **Found during:** Task 2 (Text implementation)
- **Issue:** Plan pseudocode used `TSpan::new().add(TextNode::new(...))` but svg 0.18 requires `TSpan::new(content)` and `Text::new(content)` — content is a constructor argument, not added as a child node
- **Fix:** Changed `TSpan::new()` to `TSpan::new(line.to_string())` and `SvgText::new()` to `SvgText::new("")` (empty for parent container)
- **Files modified:** src/primitives/text.rs
- **Verification:** cargo build succeeds, all 22 unit tests pass
- **Committed in:** b0377e0 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - svg crate API mismatch)
**Impact on plan:** Minimal fix required by svg 0.18 API. No scope creep. Behavior identical to plan intent.

## Issues Encountered

- Missing `examples/basic_scene.rs` file referenced in Cargo.toml — pre-existing issue unrelated to this plan. `cargo test --lib` used to run unit tests; all 22 pass.

## Next Phase Readiness

- All six primitives (Circle, Rect, Line, Arrow, Text, Bezier) are now fully implemented with builders and to_svg_element()/to_svg_parts()
- Plan 01-05 can wire all six into svg_gen dispatch — the two-pass Arrow pattern (defs before line) is ready for implementation
- Text requires Noto Sans in fontdb (already loaded in Scene::new() from plan 01-02)

---
*Phase: 01-rendering-pipeline-and-primitives*
*Completed: 2026-02-25*
