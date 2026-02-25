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
- [ ] **Phase 3: Data Visualization** - Cartesian axes, data curves, and auto-ranging coordinate mapping
- [ ] **Phase 4: GAM Visualization** - Confidence bands and animated spline fitting

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

### Phase 3: Data Visualization
**Goal**: Users can create publication-quality 2D data plots with axes, smooth curves from data points, and automatic axis scaling
**Depends on**: Phase 2
**Requirements**: DATA-01, DATA-02, DATA-03
**Success Criteria** (what must be TRUE):
  1. User can create 2D cartesian axes with tick marks, numeric labels, and a configurable axis range, rendered cleanly in the output video
  2. User can pass a `Vec<(f64, f64)>` of data points and receive a smooth curve rendered as an SVG path on the axes
  3. When the user provides data without specifying axis range, axes automatically scale to fit all data points with reasonable padding
  4. Data-space coordinates correctly map to visual-space positions -- a point at (5, 10) in data space appears at the correct location relative to the axis ticks
**Plans**: TBD

Plans:
- [ ] 03-01: TBD

### Phase 4: GAM Visualization
**Goal**: Users can render the core GAM visualization elements -- confidence bands and animated spline fits -- that no other Rust tool provides
**Depends on**: Phase 3
**Requirements**: GAM-01, GAM-02
**Success Criteria** (what must be TRUE):
  1. User can specify upper and lower bound curves and render a shaded confidence band (filled region between them) on axes
  2. User can animate a spline fitting to data -- the output video shows a curve transitioning from a flat/initial state to the fitted curve shape over time
  3. Confidence bands and spline animations compose with axes and data curves from Phase 3 -- a single scene can show axes, data curve, confidence band, and spline fit animation together
**Plans**: TBD

Plans:
- [ ] 04-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Rendering Pipeline and Primitives | 5/5 | Complete   | 2026-02-25 |
| 2. Animation Engine | 3/3 | Complete   | 2026-02-25 |
| 3. Data Visualization | 0/0 | Not started | - |
| 4. GAM Visualization | 0/0 | Not started | - |
