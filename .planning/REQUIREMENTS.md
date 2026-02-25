# Requirements: eidos

**Defined:** 2026-02-24
**Core Value:** A Rust-native way to produce beautiful, animated data visualizations with a declarative API — no Python, no GUI, just code that describes a scene and produces a video.

## v1 Requirements

### Core (Rendering Pipeline)

- [x] **CORE-01**: User can render a scene to an MP4 video file
- [x] **CORE-02**: User can configure video resolution and framerate

### Primitives

- [x] **PRIM-01**: User can add a circle with configurable fill, stroke, and opacity
- [x] **PRIM-02**: User can add a rectangle with configurable fill, stroke, and opacity
- [x] **PRIM-03**: User can add a line with configurable stroke color and width
- [x] **PRIM-04**: User can add an arrow (directed line with arrowhead) with configurable styling
- [x] **PRIM-05**: User can add a text label with configurable content, position, and size
- [x] **PRIM-06**: User can add a bezier curve/path with configurable stroke

### Animation

- [x] **ANIM-01**: User can animate any visual property (position, color, opacity, scale) between two states with easing functions
- [x] **ANIM-02**: User can compose multiple animations to run simultaneously (parallel composition)

### Data Visualization

- [x] **DATA-01**: User can create 2D cartesian axes with ticks, labels, and configurable range
- [x] **DATA-02**: User can construct a smooth curve from `Vec<(f64, f64)>` data points rendered as an SVG path
- [x] **DATA-03**: Axes auto-range to fit provided data (no manual min/max required)

### GAM Visualization

- [x] **GAM-01**: User can create a confidence band — a shaded region between two curves representing upper and lower bounds
- [ ] **GAM-02**: User can animate a spline fitting to data — a curve that transitions from a flat/initial state to the fitted curve shape

## v2 Requirements

### Output

- **OUT-01**: User can export an animated GIF
- **OUT-02**: User can export individual frames as PNG files for debugging

### Animation

- **ANIM-03**: FadeIn / FadeOut animation primitives
- **ANIM-04**: Sequential animation composition (play one after another)
- **ANIM-05**: Wait/pause primitive (hold state for N seconds)

### GAM Visualization

- **GAM-03**: Partial dependence plot as a single composite object (axes + curve + band)
- **GAM-04**: Rug plot (tick marks along axis showing raw data density)

### API

- **API-01**: `scene!{}` declarative macro DSL (thin wrapper over stable builder API)
- **API-02**: Sensible visual defaults — zero-config beautiful output

## Out of Scope

| Feature | Reason |
|---------|--------|
| LaTeX rendering | Not needed for GAM visualization focus; high complexity |
| GUI / interactive output | Code-only by design; video files are the target |
| Python bindings | The whole point is to stay in Rust |
| 3D scenes | Out of scope for statistical visualization use case |
| WASM / browser output | Video files only for v1+ |
| Real-time rendering | Offline batch rendering only |
| GPU rendering | SVG pipeline sufficient; adds C dep complexity |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1 | Complete |
| CORE-02 | Phase 1 | Complete |
| PRIM-01 | Phase 1 | Complete |
| PRIM-02 | Phase 1 | Complete |
| PRIM-03 | Phase 1 | Complete |
| PRIM-04 | Phase 1 | Complete |
| PRIM-05 | Phase 1 | Complete |
| PRIM-06 | Phase 1 | Complete |
| ANIM-01 | Phase 2 | Complete |
| ANIM-02 | Phase 2 | Complete |
| DATA-01 | Phase 3 | Complete |
| DATA-02 | Phase 3 | Complete |
| DATA-03 | Phase 3 | Complete |
| GAM-01 | Phase 4 | Complete |
| GAM-02 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 15 total
- Mapped to phases: 15
- Unmapped: 0

---
*Requirements defined: 2026-02-24*
*Last updated: 2026-02-24 after roadmap creation*
