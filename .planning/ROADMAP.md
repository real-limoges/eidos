# Roadmap: eidos

## Overview

Eidos goes from empty crate to animated GAM visualizations in four phases. Phase 1 builds the full rendering pipeline end-to-end (SVG generation, rasterization, video encoding) with all basic shape primitives. Phase 2 adds the animation engine -- property interpolation, easing, and parallel composition -- so scenes come alive. Phase 3 introduces data-aware objects (axes, curves, auto-ranging) that map data space to visual space. Phase 4 delivers the GAM-specific features that no other tool provides: confidence bands and spline fit animation.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Rendering Pipeline and Primitives** - Static scenes with styled shapes rendered to MP4 video (completed 2026-02-25)
- [x] **Phase 2: Animation Engine** - Smooth property animation with easing and parallel composition (completed 2026-02-25)
- [x] **Phase 2.5: Tech Debt Cleanup** (INSERTED) - Close v1.0 audit tech debt: dead code removal, LineState/TextState example coverage, formal Phase 1 verification record (completed 2026-02-25)
- [x] **Phase 3: Data Visualization** - Cartesian axes, data curves, and auto-ranging coordinate mapping (completed 2026-02-25)
- [x] **Phase 3.5: Dataviz Tech Debt Cleanup** (INSERTED) - Close Phase 3 audit tech debt: E2E MP4 integration test, Cargo.toml example registration, unused import removal (completed 2026-02-25)
- [x] **Phase 4: GAM Visualization** - Confidence bands and animated spline fitting (completed 2026-02-25)
- [x] **Phase 4.5: GAM Visualization Completion** (INSERTED) - Complete Phase 4 sign-off: human visual gate for gam_plot.mp4 + Axes::plot_bounds() to fix coordinate mapping contract (INT-01) (completed 2026-02-25)
- [x] **Phase 4.6: v1.0 API Ergonomics Cleanup** (INSERTED) - Public API re-exports at crate root (SceneBuilder, *State types, primitive types), remove deprecated encode_to_mp4, fix docs overclaim (completed 2026-02-25)

## Phase Details

### Phase 1: Rendering Pipeline and Primitives
**Goal**: Users can compose styled geometric primitives into a static scene and render it to an MP4 video file
**Depends on**: Nothing (first phase)
**Requirements**: CORE-01, CORE-02, PRIM-01, PRIM-02, PRIM-03, PRIM-04, PRIM-05, PRIM-06
**Success Criteria** (what must be TRUE):
  1. User can write Rust code that creates a scene with circles, rectangles, lines, arrows, text labels, and bezier curves, then call a render function that produces an MP4 file on disk
  2. Each primitive accepts fill color, stroke color, stroke width, and opacity configuration through a builder API
  3. User can set video resolution and framerate before rendering, and the output file reflects those settings
  4. The rendered video plays correctly in a standard video player (QuickTime, VLC) with all primitives visible at their specified positions and styles
**Plans**: 5 plans

Plans:
- [ ] 01-01-PLAN.md — Crate skeleton: Cargo.toml dependencies, EidosError, Color, module structure
- [ ] 01-02-PLAN.md — Scene struct and render pipeline: SVG generation, resvg rasterization, ffmpeg MP4 encoding
- [ ] 01-03-PLAN.md — Circle and Rect primitive builders with eager validation and SVG conversion
- [ ] 01-04-PLAN.md — Line, Arrow, Text, BezierPath primitive builders
- [ ] 01-05-PLAN.md — Wire all primitives into svg_gen dispatch, basic_scene example, integration tests

### Phase 2: Animation Engine
**Goal**: Users can animate any visual property between states with easing functions and compose multiple animations in parallel
**Depends on**: Phase 1
**Requirements**: ANIM-01, ANIM-02
**Success Criteria** (what must be TRUE):
  1. User can specify a start state and end state for any visual property (position, color, opacity, scale) and eidos produces a video showing smooth interpolation between them
  2. User can select from standard easing functions (linear, ease-in, ease-out, ease-in-out) and the animation curve visibly differs between choices
  3. User can compose multiple property animations to run simultaneously -- e.g., a circle moves right while fading from red to blue -- and both animations play in sync in the output video
**Plans**: 3 plans

Plans:
- [ ] 02-01-PLAN.md — Animation foundation: Easing enum, Tween<P>, and *State structs for Circle/Rect/Line/Text with CanTween derive
- [ ] 02-02-PLAN.md — Render pipeline upgrade: encode_to_mp4_animated(), Scene::render(t), render_static() backward-compat wrapper
- [ ] 02-03-PLAN.md — Wire-up: fix Phase 1 callers, animated_scene example, Phase 2 integration tests + human verification

### Phase 2.5: Tech Debt Cleanup
**Goal**: Resolve the 3 low-severity tech debt items surfaced by the v1.0 audit before Phase 3 begins
**Depends on**: Phase 2
**Requirements**: None (housekeeping — no new requirements)
**Gap Closure:** Closes tech debt from audit
**Success Criteria** (what must be TRUE):
  1. `svg_gen::encode_to_mp4` is removed or deprecated with a clear doc comment explaining why `encode_to_mp4_animated` is the preferred path
  2. `LineState` and `TextState` each appear in at least one integration test and the `animated_scene` example (or a new example)
  3. `01-VERIFICATION.md` status is updated to `passed` after formal visual playback confirmation of `basic_scene.mp4`
**Plans**: 2 plans

Plans:
- [ ] 02.5-01-PLAN.md — Deprecate encode_to_mp4, add LineState/TextState to animated_scene example and integration tests
- [ ] 02.5-02-PLAN.md — Regenerate basic_scene.mp4, human visual confirmation, update 01-VERIFICATION.md to passed

### Phase 3: Data Visualization
**Goal**: Users can create publication-quality 2D data plots with axes, smooth curves from data points, and automatic axis scaling
**Depends on**: Phase 2
**Requirements**: DATA-01, DATA-02, DATA-03
**Success Criteria** (what must be TRUE):
  1. User can create 2D cartesian axes with tick marks, numeric labels, and a configurable axis range, rendered cleanly in the output video
  2. User can pass a `Vec<(f64, f64)>` of data points and receive a smooth curve rendered as an SVG path on the axes
  3. When the user provides data without specifying axis range, axes automatically scale to fit all data points with reasonable padding
  4. Data-space coordinates correctly map to visual-space positions -- a point at (5, 10) in data space appears at the correct location relative to the axis ticks
**Plans**: 3 plans

Plans:
- [ ] 03-01-PLAN.md — DataCurve struct and Catmull-Rom spline algorithm (src/dataviz/ module foundation)
- [ ] 03-02-PLAN.md — Axes struct with Heckbert tick generation, coordinate mapping, auto-range, and to_primitives()
- [ ] 03-03-PLAN.md — Wire dataviz into lib.rs and scene.rs, data_plot example, integration tests, human verification

### Phase 3.5: Dataviz Tech Debt Cleanup
**Goal**: Resolve the 3 tech debt items surfaced by the v1.0 audit for Phase 3 before Phase 4 begins
**Depends on**: Phase 3
**Requirements**: None (housekeeping — DATA-01, DATA-02, DATA-03, CORE-01 already satisfied)
**Gap Closure:** Closes tech debt from v1.0 audit
**Success Criteria** (what must be TRUE):
  1. `tests/data_viz.rs` contains a `dataviz_render_produces_mp4` test that calls `scene.add_axes()`, `scene.render_static()`, and asserts a valid MP4 is produced on disk
  2. `data_plot` has an explicit `[[example]]` block in `Cargo.toml`, consistent with `basic_scene` and `animated_scene`
  3. The unused `AxisRange` import warning in `tests/data_viz.rs:5` is resolved (import removed or used)
**Plans**: 1 plan

Plans:
- [ ] 03.5-01-PLAN.md — Fix tests/data_viz.rs (remove unused AxisRange import, add dataviz_render_produces_mp4 E2E test) and add data_plot [[example]] to Cargo.toml

### Phase 4: GAM Visualization
**Goal**: Users can render the core GAM visualization elements -- confidence bands and animated spline fits -- that no other Rust tool provides
**Depends on**: Phase 3
**Requirements**: GAM-01, GAM-02
**Success Criteria** (what must be TRUE):
  1. User can specify upper and lower bound curves and render a shaded confidence band (filled region between them) on axes
  2. User can animate a spline fitting to data -- the output video shows a curve transitioning from a flat/initial state to the fitted curve shape over time
  3. Confidence bands and spline animations compose with axes and data curves from Phase 3 -- a single scene can show axes, data curve, confidence band, and spline fit animation together
**Plans**: 3 plans

Plans:
- [ ] 04-01-PLAN.md — Shared Catmull-Rom helper (spline.rs) + ConfidenceBand struct + Axes::add_band() integration
- [ ] 04-02-PLAN.md — SplineFit struct with frame-time morphing: left-to-right reveal + y-value interpolation from mean_y
- [ ] 04-03-PLAN.md — Wire lib.rs pub use, gam_plot example, integration tests, human visual confirmation

### Phase 4.5: GAM Visualization Completion
**Goal**: Complete Phase 4 milestone sign-off — confirm gam_plot.mp4 visual output and fix the coordinate mapping contract between Axes and SplineFit
**Depends on**: Phase 4
**Requirements**: None (GAM-01 and GAM-02 already satisfied; this closes the human verification gate and INT-01 design gap)
**Gap Closure:** Closes tech debt from v1.0 audit: Phase 4 human_needed verification status, INT-01 fragile coordinate contract
**Success Criteria** (what must be TRUE):
  1. 04-VERIFICATION.md status is updated to `passed` after human visual confirmation of gam_plot.mp4 (confidence band shading, yellow spline reveal, y-morph from flat)
  2. `Axes::plot_bounds()` public method exists and returns tick-adjusted `(x_min, x_max, y_min, y_max)` — callers no longer need to replicate internal Axes coordinate mapping
  3. gam_plot.rs and gam_viz.rs tests use `axes.plot_bounds()` to compute visual_pts instead of hardcoded helper functions
**Plans**: 1 plan

Plans:
- [ ] 04.5-01-PLAN.md — Human visual gate (gam_plot.mp4 confirmation) + Axes::plot_bounds() + update gam_plot example and gam_viz tests to use it

### Phase 4.6: v1.0 API Ergonomics Cleanup
**Goal**: Polish the v1.0 public API surface — re-export all user-facing types at the crate root, remove the deprecated encode_to_mp4 function, fix documentation inaccuracy
**Depends on**: Phase 4.5
**Requirements**: None (housekeeping — all v1 requirements already satisfied)
**Gap Closure:** Closes tech debt from v1.0 audit: public API re-export gaps, deprecated dead code, docs overclaim
**Success Criteria** (what must be TRUE):
  1. Users can write `use eidos::{SceneBuilder, CircleState, RectState, LineState, TextState, Circle, Rect, Line, Arrow, Text, Bezier, Primitive}` — all user-facing types at crate root
  2. `cargo build --all-targets` produces zero warnings (encode_to_mp4 removed, no dead_code allow needed)
  3. 03-03-SUMMARY.md correctly states "11 integration tests"
**Plans**: 1 plan

Plans:
- [ ] 04.6-01-PLAN.md — Add pub use re-exports to src/lib.rs, remove encode_to_mp4 from svg_gen.rs, fix 03-03-SUMMARY.md

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 2.5 -> 3 -> 3.5 -> 4 -> 4.5 -> 4.6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Rendering Pipeline and Primitives | 5/5 | Complete   | 2026-02-25 |
| 2. Animation Engine | 3/3 | Complete   | 2026-02-25 |
| 2.5. Tech Debt Cleanup | 2/2 | Complete    | 2026-02-25 |
| 3. Data Visualization | 3/3 | Complete   | 2026-02-25 |
| 3.5. Dataviz Tech Debt Cleanup | 1/1 | Complete   | 2026-02-25 |
| 4. GAM Visualization | 3/3 | Complete   | 2026-02-25 |
| 4.5. GAM Visualization Completion | 1/1 | Complete   | 2026-02-25 |
| 4.6. v1.0 API Ergonomics Cleanup | 0/1 | Complete    | 2026-02-25 |
