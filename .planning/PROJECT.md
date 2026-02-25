# eidos

## What This Is

A Manim-inspired Rust library for programmatic data visualization and animation. Users describe scenes declaratively — what objects appear, not when or how — and eidos handles animation, interpolation, and rendering to video. Built initially for GAM visualizations (spline fits, partial dependence plots, confidence bands), with a general-purpose API designed for broader adoption.

## Core Value

A Rust-native way to produce beautiful, animated data visualizations with a declarative API — no Python, no GUI, just code that describes a scene and produces a video.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Declarative scene composition API (describe what, not when/how)
- [ ] First-class mathematical/statistical objects: curves, axes, confidence bands, labels
- [ ] Smooth animation between states (interpolation, easing)
- [ ] Video output (MP4/GIF) via SVG → rasterize → encode pipeline
- [ ] GAM-specific primitives: spline fits, partial dependence plots, shaded confidence bands
- [ ] Composable scene elements (combine objects into groups, layer scenes)

### Out of Scope

- GUI or interactive editor — code-only, by design
- LaTeX rendering — not needed for GAM visualization focus
- Real-time/interactive output — video files only for v1
- Python bindings — the whole point is to stay in Rust

## Context

- Crate name: `eidos` (fresh skeleton, no dependencies yet, Rust 2024 edition)
- Inspired by Manim (Python) / 3Blue1Brown aesthetic — clean vector look, smooth motion
- Primary use case: office presentations of GAM model outputs
- Rendering philosophy: SVG per frame → rasterize (tiny-skia/resvg) → encode with ffmpeg
- Target: start as personal tooling, generalize API as patterns emerge

## Constraints

- **Language**: Rust — no Python, no FFI to Python ecosystem
- **Output**: Video files (MP4/GIF) — not interactive, not browser-based for v1
- **API**: Declarative macro/builder — `scene! { }` style, not imperative sequencing

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| SVG → rasterize → ffmpeg pipeline | Clean vector look, composable, no GPU dependency for v1 | — Pending |
| Declarative "describe scene, library animates" API | Matches user mental model, differentiates from raw drawing libs | — Pending |
| GAM primitives as first-class objects | Drives initial design with concrete use cases before generalizing | — Pending |

---
*Last updated: 2026-02-24 after initialization*
