# Roadmap: eidos

## Milestones

- ✅ **v1.0 — Rendering Foundation** — Phases 1–4.6 (shipped 2026-02-25)
- ✅ **v1.1 — 3D Surface Visualization** — Phases 5–9.1 (shipped 2026-02-26)
- 🚧 **v1.2 — API Polish & Ergonomics** — Phases 10–12 (in progress)

## Phases

<details>
<summary>✅ v1.0 — Phases 1–4.6 — SHIPPED 2026-02-25</summary>

- [x] Phase 1: Rendering Pipeline and Primitives (5/5 plans) — completed 2026-02-25
- [x] Phase 2: Animation Engine (3/3 plans) — completed 2026-02-25
- [x] Phase 2.5: Tech Debt Cleanup (2/2 plans) — completed 2026-02-25
- [x] Phase 3: Data Visualization (3/3 plans) — completed 2026-02-25
- [x] Phase 3.5: Dataviz Tech Debt Cleanup (1/1 plan) — completed 2026-02-25
- [x] Phase 4: GAM Visualization (3/3 plans) — completed 2026-02-25
- [x] Phase 4.5: GAM Visualization Completion (1/1 plan) — completed 2026-02-25
- [x] Phase 4.6: v1.0 API Ergonomics Cleanup (1/1 plan) — completed 2026-02-25

Full phase details: `.planning/milestones/v1.0-ROADMAP.md`

</details>

<details>
<summary>✅ v1.1 — 3D Surface Visualization — Phases 5–9.1 — SHIPPED 2026-02-26</summary>

- [x] Phase 5: Camera and Projection Foundation (3/3 plans) — completed 2026-02-25
- [x] Phase 6: Static 3D Surface Rendering (3/3 plans) — completed 2026-02-26
- [x] Phase 7: Surface and Camera Animation (2/2 plans) — completed 2026-02-26
- [x] Phase 8: Scatter Points (2/2 plans) — completed 2026-02-26
- [x] Phase 9: v1.1 Integration Test Coverage (1/1 plan) — completed 2026-02-26
- [x] Phase 9.1: v1.1 SUMMARY Schema and Doc Fixes (1/1 plan) — completed 2026-02-26

Full phase details: `.planning/milestones/v1.1-ROADMAP.md`

</details>

### 🚧 v1.2 — API Polish & Ergonomics (In Progress)

**Milestone Goal:** Eliminate the most common friction points in the eidos API — verbose animation state construction, manual coordinate math, and unpredictable `?` in builder chains.

- [x] **Phase 10: Infallible Builders** — All primitive builder methods return `Self` with clamped values; no `?` required (completed 2026-02-26)
- [x] **Phase 11: State and Tween Ergonomics** — State types accept `Color` directly; `Tween` fluent builder API (1 plan) (completed 2026-02-26)
- [x] **Phase 12: Coordinate Mapping** — `Axes::map_point()` data-to-pixel coordinate helper (completed 2026-02-26)

## Phase Details

### Phase 10: Infallible Builders
**Goal**: All primitive builder methods are infallible — users can chain `.opacity()`, `.stroke()`, `.font_size()` without wrapping results in `?`
**Depends on**: Phase 9.1 (v1.1 complete)
**Requirements**: API-01
**Success Criteria** (what must be TRUE):
  1. User can chain `.opacity(2.0)` on any primitive and get a valid object back — value is clamped to 1.0, no `Err` returned
  2. User can chain `.stroke(-5.0)` on any primitive and get a valid object back — value is clamped to 0.0
  3. User can chain `.font_size(0.0)` on `Text` and get a valid object back — value is clamped to a minimum positive size
  4. A builder chain like `Circle::new(...).opacity(0.5).stroke(2.0)` compiles and runs without any `?` or `.unwrap()`
  5. All existing examples and tests that used `?` on builder methods compile after the signature change
**Plans**: 2 plans
Plans:
- [ ] 10-01-PLAN.md — Change all 6 primitive types to return Self from opacity/stroke/font_size/line_height/stroke_width builders
- [ ] 10-02-PLAN.md — Remove all .unwrap()/.expect()/? from builder call sites in dataviz, examples, and tests

### Phase 11: State and Tween Ergonomics
**Goal**: Users can construct animation states using `Color` values directly and build `Tween` instances with a fluent method chain instead of struct literals
**Depends on**: Phase 10
**Requirements**: ERGO-01, ERGO-02
**Success Criteria** (what must be TRUE):
  1. User can write `CircleState { fill: Color::rgb(255, 0, 0), opacity: 1.0, .. }` (or equivalent constructor) without specifying `fill_r`, `fill_g`, `fill_b` separately
  2. Same `Color`-based construction works for `RectState`, `LineState`, and `TextState`
  3. User can write `Tween::builder().from(s1).to(s2).start_at(0.0).over(1.0).build()` (or equivalent fluent chain) instead of a struct literal
  4. Existing code that used struct literal `Tween { .. }` either still compiles or has a clear migration path
**Plans**: 1 plan
Plans:
- [ ] 11-01-PLAN.md — Add Color-based State constructors + TweenBuilder fluent API + migrate all callers

### Phase 12: Coordinate Mapping
**Goal**: Users can convert data-space coordinates to pixel coordinates via a single method call on `Axes`, eliminating manual transform math in examples and user code
**Depends on**: Phase 11
**Requirements**: COORD-01
**Success Criteria** (what must be TRUE):
  1. User can call `axes.map_point(data_x, data_y)` and receive `(pixel_x, pixel_y)` as `(f64, f64)`
  2. The returned pixel coordinates match what the manual transform in existing examples produces for the same input
  3. `map_point` is accessible on an `Axes` value after calling `scene.add_axes(...)` — no internal type required
**Plans**: 1 plan
Plans:
- [ ] 12-01-PLAN.md — Add map_point method to Axes and migrate all manual coordinate transforms

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Rendering Pipeline and Primitives | v1.0 | 5/5 | Complete | 2026-02-25 |
| 2. Animation Engine | v1.0 | 3/3 | Complete | 2026-02-25 |
| 2.5. Tech Debt Cleanup | v1.0 | 2/2 | Complete | 2026-02-25 |
| 3. Data Visualization | v1.0 | 3/3 | Complete | 2026-02-25 |
| 3.5. Dataviz Tech Debt Cleanup | v1.0 | 1/1 | Complete | 2026-02-25 |
| 4. GAM Visualization | v1.0 | 3/3 | Complete | 2026-02-25 |
| 4.5. GAM Visualization Completion | v1.0 | 1/1 | Complete | 2026-02-25 |
| 4.6. v1.0 API Ergonomics Cleanup | v1.0 | 1/1 | Complete | 2026-02-25 |
| 5. Camera and Projection Foundation | v1.1 | 3/3 | Complete | 2026-02-25 |
| 6. Static 3D Surface Rendering | v1.1 | 3/3 | Complete | 2026-02-26 |
| 7. Surface and Camera Animation | v1.1 | 2/2 | Complete | 2026-02-26 |
| 8. Scatter Points | v1.1 | 2/2 | Complete | 2026-02-26 |
| 9. v1.1 Integration Test Coverage | v1.1 | 1/1 | Complete | 2026-02-26 |
| 9.1. v1.1 SUMMARY Schema and Doc Fixes | v1.1 | 1/1 | Complete | 2026-02-26 |
| 10. Infallible Builders | 2/2 | Complete    | 2026-02-26 | - |
| 11. State and Tween Ergonomics | 1/1 | Complete    | 2026-02-26 | - |
| 12. Coordinate Mapping | 1/1 | Complete   | 2026-02-26 | - |
