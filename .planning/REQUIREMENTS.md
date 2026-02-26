# Requirements: eidos

**Defined:** 2026-02-25
**Core Value:** A Rust-native way to produce beautiful, animated data visualizations with a declarative API — no Python, no GUI, just code that describes a scene and produces a video.

## v1.1 Requirements

Requirements for the 3D Surface Visualization milestone. Each maps to roadmap phases.

### Surface Rendering

- [x] **SURF-01**: User can create a 3D surface plot from a regular grid of (x, y, z) data with a configurable camera viewpoint
- [x] **SURF-02**: User can render the surface as a wireframe mesh (depth-sorted projected edges)
- [x] **SURF-03**: User can render the surface as a shaded mesh with a z-height color gradient
- [x] **SURF-04**: User can add 3D cartesian axes with projected tick marks and labels to a surface plot

### Scatter

- [ ] **SCAT-01**: User can add (x, y, z) scatter points to a 3D plot, rendered with depth-based opacity
- [ ] **SCAT-02**: User can animate scatter points fading in over a specified time range

### Animation

- [x] **ANIM-01**: User can animate the surface morphing from flat to fitted shape over a specified time range
- [x] **ANIM-02**: User can animate the camera orbiting around the surface (azimuth sweep) over a specified time range

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

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SURF-01 | Phase 5 | Complete (05-01) |
| SURF-02 | Phase 6 | Complete |
| SURF-03 | Phase 6 | Complete |
| SURF-04 | Phase 6 | Complete |
| SCAT-01 | Phase 8 | Pending |
| SCAT-02 | Phase 8 | Pending |
| ANIM-01 | Phase 7 | Complete |
| ANIM-02 | Phase 7 | Complete |

**Coverage:**
- v1.1 requirements: 8 total
- Mapped to phases: 8
- Unmapped: 0 ✓

---
*Requirements defined: 2026-02-25*
*Last updated: 2026-02-25 — traceability populated after roadmap creation*
