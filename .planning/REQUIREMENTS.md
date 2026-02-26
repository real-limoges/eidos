# Requirements: eidos

**Defined:** 2026-02-26
**Core Value:** A Rust-native way to produce beautiful, animated data visualizations with a declarative API — no Python, no GUI, just code that describes a scene and produces a video.

## v1.2 Requirements

Requirements for the API Polish & Ergonomics milestone. Each maps to roadmap phases.

### ERGO — State & Animation Ergonomics

- [x] **ERGO-01**: User can construct `CircleState`, `RectState`, `LineState`, and `TextState` using a `Color` value directly — no separate `fill_r`, `fill_g`, `fill_b` f64 channel fields required
- [x] **ERGO-02**: User can build a `Tween` using a fluent builder API (`.from()` / `.to()` / `.start_at()` / `.over()` / `.easing()`) instead of struct literal initialization

### COORD — Coordinate Mapping

- [x] **COORD-01**: User can call `axes.map_point(data_x: f64, data_y: f64) -> (f64, f64)` to convert data-space coordinates to pixel coordinates without writing manual coordinate transform math

### API — API Consistency

- [x] **API-01**: All primitive builder methods that currently return `Result<Self>` (`.opacity()`, `.stroke()`, `.font_size()`) are changed to return `Self` — invalid values are clamped to valid ranges — so no builder chain ever requires `?`

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Rendering

- **REND-01**: Contour projection onto the base plane
- **REND-02**: Directional lighting / Lambertian shading
- **REND-03**: Unstructured triangle mesh input (non-regular grids)

### Camera

- **CAM-01**: Full quaternion SLERP camera interpolation for pole-region orbits (>180° arcs)
- **CAM-02**: Elevation swing animation as complement to azimuth orbit

## Out of Scope

| Feature | Reason |
|---------|--------|
| Interactive rotation | Requires event loop — fundamentally incompatible with video-only headless architecture |
| GPU rendering path | Breaks the SVG pipeline; only justified if software performance wall cannot be solved |
| Python bindings | The whole point is to stay in Rust |
| Real-time / streaming output | Video files only, by design |
| Named parameters for primitive constructors | Rust doesn't support named args; struct literal already readable with IDE support |
| `SurfacePlot::from_grid()` nested input | Row-major flat Vec is idiomatic Rust; nested vec adds allocation without clarity benefit |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| ERGO-01 | Phase 11 | Complete |
| ERGO-02 | Phase 11 | Complete |
| COORD-01 | Phase 12 | Complete |
| API-01 | Phase 10 | Complete |

**Coverage:**
- v1.2 requirements: 4 total
- Mapped to phases: 4
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-26*
*Last updated: 2026-02-26 — traceability updated after v1.2 roadmap creation*
