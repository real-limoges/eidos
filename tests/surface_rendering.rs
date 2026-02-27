// tests/surface_rendering.rs
//! Integration tests for Phase 9: surface render_static() flow, render modes, and axis assertions.
//! Closes test gaps from v1.1 milestone audit (SURF-01, SURF-02, SURF-04).

use eidos::{Camera, Primitive, RenderMode, Scene, SurfacePlot};
use std::path::PathBuf;

fn ffmpeg_available() -> bool {
    std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok()
}

fn temp_mp4(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}

fn make_paraboloid(n: usize) -> SurfacePlot {
    let mut xs = Vec::with_capacity(n * n);
    let mut ys = Vec::with_capacity(n * n);
    let mut zs = Vec::with_capacity(n * n);
    for r in 0..n {
        for c in 0..n {
            let x = r as f64 / (n - 1) as f64;
            let y = c as f64 / (n - 1) as f64;
            xs.push(x);
            ys.push(y);
            zs.push(x * x + y * y);
        }
    }
    SurfacePlot::new(xs, ys, zs, n, n)
}

/// SURF-01 / audit gap: render_static() with SurfacePlot was not exercised end-to-end.
/// E2E Flow 1: SurfacePlot::new() -> scene.render_static() -> MP4 > 1000 bytes.
#[test]
fn static_surface_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    let plot = make_paraboloid(8);
    let camera = Camera::new(45.0, 30.0, 3.0);
    let out = temp_mp4("surface_static_test.mp4");
    let scene = Scene::new(800, 600, 24).expect("valid scene").duration(1.0);

    scene
        .render_static(
            |s| {
                s.add_surface(&plot, &camera, (800, 600));
            },
            &out,
        )
        .expect("static surface render should succeed");

    let meta = std::fs::metadata(&out).expect("output MP4 must exist");
    assert!(
        meta.len() > 1000,
        "output file too small: {} bytes",
        meta.len()
    );
    std::fs::remove_file(&out).ok();
}

/// SURF-02 / audit gap: RenderMode::Wireframe not exercised in any integration test.
#[test]
fn wireframe_surface_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    let plot = make_paraboloid(8).render_mode(RenderMode::Wireframe);
    let camera = Camera::new(45.0, 30.0, 3.0);
    let out = temp_mp4("surface_wireframe_test.mp4");
    let scene = Scene::new(800, 600, 24).expect("valid scene").duration(1.0);

    scene
        .render_static(
            |s| {
                s.add_surface(&plot, &camera, (800, 600));
            },
            &out,
        )
        .expect("wireframe render should succeed");

    let meta = std::fs::metadata(&out).expect("output MP4 must exist");
    assert!(
        meta.len() > 1000,
        "output file too small: {} bytes",
        meta.len()
    );
    std::fs::remove_file(&out).ok();
}

/// SURF-02 / audit gap: RenderMode::ShadedWireframe not exercised in any integration test.
#[test]
fn shaded_wireframe_surface_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    let plot = make_paraboloid(8).render_mode(RenderMode::ShadedWireframe);
    let camera = Camera::new(45.0, 30.0, 3.0);
    let out = temp_mp4("surface_shaded_wireframe_test.mp4");
    let scene = Scene::new(800, 600, 24).expect("valid scene").duration(1.0);

    scene
        .render_static(
            |s| {
                s.add_surface(&plot, &camera, (800, 600));
            },
            &out,
        )
        .expect("shaded wireframe render should succeed");

    let meta = std::fs::metadata(&out).expect("output MP4 must exist");
    assert!(
        meta.len() > 1000,
        "output file too small: {} bytes",
        meta.len()
    );
    std::fs::remove_file(&out).ok();
}

/// SURF-04 / audit gap: axis primitive presence not asserted in integration context.
/// No ffmpeg required -- pure to_primitives() assertion from external consumer.
#[test]
fn to_primitives_contains_face_and_axis_primitives() {
    let plot = make_paraboloid(4);
    let camera = Camera::new(45.0, 30.0, 3.0);
    let prims = plot.to_primitives(&camera, (800, 600));

    let bezier_count = prims
        .iter()
        .filter(|p| matches!(p, Primitive::Bezier(_)))
        .count();
    let non_bezier_count = prims
        .iter()
        .filter(|p| !matches!(p, Primitive::Bezier(_)))
        .count();

    assert!(
        bezier_count >= 1,
        "to_primitives should produce Bezier face primitives; got 0"
    );
    assert!(
        non_bezier_count > 0,
        "to_primitives should produce axis Line/Text primitives; got 0"
    );
}
