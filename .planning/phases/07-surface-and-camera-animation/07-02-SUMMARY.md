---
phase: 07-surface-and-camera-animation
plan: 02
subsystem: dataviz
tags: [animation, surface, camera, integration-test, mp4, rust]

# Dependency graph
requires:
  - phase: 07-01
    provides: SurfacePlot::animate_fit, to_primitives_at, animate_camera_azimuth, camera_at
  - phase: 06-static-3d-surface-rendering
    provides: SurfacePlot::to_primitives, Camera, painter's algorithm
provides:
  - SceneBuilder::add_surface_at() — animated counterpart to add_surface
  - tests/surface_animation.rs — three integration tests proving end-to-end animated MP4 output
affects: [scene.rs, integration-tests, ANIM-01, ANIM-02]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "add_surface_at thin-wrapper pattern: delegates directly to to_primitives_at (mirrors add_surface -> to_primitives)"
    - "ffmpeg guard in integration tests: graceful skip when ffmpeg unavailable"
    - "Per-frame Camera::new() inside render closure: resolves animated azimuth at t_secs via camera_at()"

key-files:
  created:
    - tests/surface_animation.rs
  modified:
    - src/scene.rs

key-decisions:
  - "add_surface_at signature matches add_surface but adds t_secs parameter — consistent API shape"
  - "camera_orbit_only test uses add_surface (not add_surface_at) to verify static surface still works during orbit — validates backward compatibility"
  - "All three tests exercise distinct animation-combination slices: both / morph-only / orbit-only"

# Metrics
duration: ~2 min
completed: 2026-02-25
---

# Phase 7 Plan 02: Surface and Camera Animation (SceneBuilder Integration) Summary

**SceneBuilder::add_surface_at() wired to to_primitives_at() and proven end-to-end by three integration tests that render morphing+orbiting surfaces to valid MP4 files**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-02-26T02:41:30Z
- **Completed:** 2026-02-26T02:42:52Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Added `SceneBuilder::add_surface_at()` to `src/scene.rs` — thin wrapper delegating to `SurfacePlot::to_primitives_at(camera, viewport, t_secs)`
- Added `add_surface_at_produces_primitives` unit test in scene.rs tests module (uses `Easing::Linear` via `crate::animation::Easing`)
- Created `tests/surface_animation.rs` with three integration tests:
  - `surface_animation_renders_to_mp4`: ANIM-01 + ANIM-02 combined (animate_fit + animate_camera_azimuth, 3-second MP4)
  - `surface_morph_only_renders_to_mp4`: ANIM-01 alone with static Camera (2-second MP4)
  - `camera_orbit_only_renders_to_mp4`: ANIM-02 alone with static surface via add_surface (2-second MP4)
- All three integration tests produce MP4 files > 1000 bytes and clean up after themselves
- Full test suite (115 lib + 3 integration + 2 doctests) passes — 0 regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: add_surface_at() + unit test in scene.rs** - `1dc7db0` (feat)
2. **Task 2: tests/surface_animation.rs integration tests** - `8a25522` (feat)

## Files Created/Modified

- `/Users/reallimoges/repositories/eidos/src/scene.rs` — Added add_surface_at() method and add_surface_at_produces_primitives unit test; added `use crate::animation::Easing` in test module
- `/Users/reallimoges/repositories/eidos/tests/surface_animation.rs` — Three integration tests: combined animation, morph-only, orbit-only

## Decisions Made

- `add_surface_at` mirrors `add_surface` in structure — same parameter order with `t_secs` appended as final parameter
- `camera_orbit_only_renders_to_mp4` uses `add_surface` (not `add_surface_at`) to explicitly verify backward-compatible rendering while camera orbits
- Three separate test functions rather than one combined: each proves a distinct capability slice and gives clearer failure attribution

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - both tasks compiled and passed on first attempt.

## User Setup Required

None - no external service configuration required.

## Requirements Satisfied

- **ANIM-01**: `animate_fit + to_primitives_at` chain exercised end-to-end in `surface_animation_renders_to_mp4` and `surface_morph_only_renders_to_mp4`
- **ANIM-02**: `animate_camera_azimuth + camera_at + Camera::new` chain exercised in `surface_animation_renders_to_mp4` and `camera_orbit_only_renders_to_mp4`

---
*Phase: 07-surface-and-camera-animation*
*Completed: 2026-02-25*

## Self-Check: PASSED

- src/scene.rs: FOUND
- tests/surface_animation.rs: FOUND
- commit 1dc7db0: FOUND
- commit 8a25522: FOUND
