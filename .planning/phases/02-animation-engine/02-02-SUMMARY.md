---
phase: 02-animation-engine
plan: "02"
subsystem: rendering
tags: [rust, ffmpeg, svg, resvg, tiny-skia, animation, per-frame, streaming]

# Dependency graph
requires:
  - phase: 01-rendering-pipeline-and-primitives
    provides: encode_to_mp4(), rasterize_frame(), Scene struct with fontdb Arc
  - phase: 02-animation-engine
    plan: "01"
    provides: Tween<P> and State structs needing a per-frame call site
provides:
  - encode_to_mp4_animated() in svg_gen.rs — per-frame closure, streaming to ffmpeg stdin
  - Scene::render(Fn(&mut SceneBuilder, f64)) — per-frame with scene time in seconds
  - Scene::render_static(Fn(&mut SceneBuilder)) — Phase 1 backward-compat wrapper
affects:
  - 02-03 (callers of render() must update to two-argument closure)
  - All future animation plans using Scene::render() with Tween::value_at(t)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Streaming frame pipeline: frame closure called per index, bytes written to ffmpeg stdin immediately"
    - "Arc::clone once before frame loop avoids per-frame fontdb re-init cost"
    - "render_static() wraps render() with |s, _t| closure for backward compatibility"

key-files:
  created: []
  modified:
    - src/svg_gen.rs
    - src/scene.rs

key-decisions:
  - "encode_to_mp4_animated takes F: Fn(u64) -> Result<Vec<u8>, EidosError> — frame index, not time; Scene::render() computes t_secs from index so callers work with seconds"
  - "render_static() delegates to render() with _t ignored — Phase 1 callers (basic_scene, integration test) will be updated in 02-03 instead of here"
  - "fontdb Arc cloned once before the frame closure, not inside it — O(1) clone cost total"

patterns-established:
  - "Per-frame render pattern: encode_to_mp4_animated(|frame_idx| { compute t_secs; build scene; rasterize }, ...)"
  - "Streaming over buffering: write each frame to stdin immediately rather than accumulating Vec<Vec<u8>>"

requirements-completed: [ANIM-01, ANIM-02]

# Metrics
duration: 2min
completed: 2026-02-25
---

# Phase 2 Plan 02: Animated Render Pipeline Summary

**Per-frame encode pipeline via encode_to_mp4_animated() and Scene::render(Fn(&mut SceneBuilder, f64)) with streaming ffmpeg stdin — zero Vec accumulation**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-25T00:26:19Z
- **Completed:** 2026-02-25T00:28:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added `encode_to_mp4_animated()` to svg_gen.rs — calls frame closure once per frame, streams RGBA bytes directly to ffmpeg stdin
- Updated `Scene::render()` from `Fn(&mut SceneBuilder)` to `Fn(&mut SceneBuilder, f64)` — f64 is scene time in seconds
- Added `Scene::render_static()` as a Phase 1 backward-compatible wrapper that ignores the time parameter
- fontdb Arc cloned once before the frame loop (not per-frame), keeping rasterization cost independent of frame count

## Task Commits

Each task was committed atomically:

1. **Task 1: Add encode_to_mp4_animated() to svg_gen.rs** - `ca5747d` (feat)
2. **Task 2: Update Scene::render() signature and add render_static()** - `6633798` (feat)

## Files Created/Modified
- `src/svg_gen.rs` - Added `encode_to_mp4_animated<F>(frame_fn, total_frames, width, height, fps, output_path)` after existing `encode_to_mp4`
- `src/scene.rs` - Replaced static `render(Fn(&mut SceneBuilder))` with animated `render(Fn(&mut SceneBuilder, f64))`; added `render_static()` wrapper

## Decisions Made
- `encode_to_mp4_animated` receives frame index (u64), not scene time — Scene::render() is responsible for computing `t_secs = frame_idx as f64 / fps as f64`, keeping svg_gen.rs independent of fps semantics
- `render_static()` calls `self.render(|s, _t| build_scene(s), output_path)` — single delegation, no code duplication
- fontdb is cloned via `Arc::clone` once before the closure capture, not inside the closure (which would clone once per frame on Arc dereference)

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None. The lib (`--lib`) compiled cleanly after both changes. As expected by the plan, `cargo test` produces two errors:
- `examples/basic_scene.rs` — closure takes 1 argument, expected 2
- `tests/integration.rs` — closure takes 1 argument, expected 2

Both are expected and will be fixed in plan 02-03.

## Compile Status

- `cargo build --lib`: PASS
- `cargo test` (full): 2 compile errors in examples/basic_scene and tests/integration (expected, per plan)

## Next Phase Readiness
- `encode_to_mp4_animated()` and `Scene::render()` are in place — plan 02-03 can now update callers to use the `|s, t| {}` signature and call `Tween::value_at(t)` inside the closure
- No blockers

---
*Phase: 02-animation-engine*
*Completed: 2026-02-25*
