// tests/scatter_points.rs
//! Integration tests for Phase 8: scatter points overlaid on 3D surface.

use eidos::{Camera, Color, Easing, ScatterPlot, Scene, SurfacePlot};
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

/// Build a paraboloid: z = x^2 + y^2 over an n×n grid.
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

/// SCAT-01: Static scatter points (no animation) render on a static surface -> valid MP4.
/// Points should appear at correct positions overlaid on the surface, depth-sorted.
#[test]
fn static_scatter_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    let plot = make_paraboloid(8);
    // Scatter points at corners and center of the data domain [0,1]x[0,1]x[0,2]
    let points = vec![
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 1.0),
        (0.0, 1.0, 1.0),
        (1.0, 1.0, 2.0),
        (0.5, 0.5, 0.5),
    ];
    let scatter = ScatterPlot::new(points, plot.data_extents())
        .with_color(Color::rgb(255, 80, 40))
        .with_radius(5.0);

    let camera = Camera::new(45.0, 30.0, 3.0);
    let out = temp_mp4("scatter_static_test.mp4");

    let scene = Scene::new(800, 600, 24).expect("valid scene").duration(1.0); // 1 second static

    scene
        .render(
            |s, t_secs| {
                s.add_surface(&plot, &camera, (800, 600));
                s.add_scatter(&scatter, &camera, (800, 600), t_secs);
            },
            &out,
        )
        .expect("static scatter render should succeed");

    let meta = std::fs::metadata(&out).expect("output MP4 must exist");
    assert!(
        meta.len() > 1000,
        "output file too small: {} bytes",
        meta.len()
    );
    std::fs::remove_file(&out).ok();
}

/// SCAT-02: Animated scatter (fade-in) renders on an animated surface -> valid MP4.
/// Surface morphs 0-3s, scatter fades in 3-5s. Total 5s video.
#[test]
fn animated_scatter_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    let plot = make_paraboloid(8).animate_fit(0.0, 3.0, Easing::EaseOut);

    let points = vec![
        (0.25, 0.25, 0.125),
        (0.75, 0.25, 0.625),
        (0.25, 0.75, 0.625),
        (0.75, 0.75, 1.125),
        (0.5, 0.5, 0.5),
        (0.1, 0.9, 0.82),
        (0.9, 0.1, 0.82),
    ];
    let scatter = ScatterPlot::new(points, plot.data_extents())
        .with_color(Color::rgb(255, 120, 50))
        .animate_fade(3.0, 5.0); // fades in after surface morph finishes

    let out = temp_mp4("scatter_animated_test.mp4");

    let scene = Scene::new(800, 600, 24).expect("valid scene").duration(5.0);

    let elevation_deg = 30.0_f64;
    let distance = 3.0_f64;

    scene
        .render(
            |s, t_secs| {
                let azimuth = plot.camera_at(t_secs).unwrap_or(45.0);
                let camera = Camera::new(azimuth, elevation_deg, distance);
                s.add_surface_at(&plot, &camera, (800, 600), t_secs);
                s.add_scatter_at(&scatter, &camera, (800, 600), t_secs);
            },
            &out,
        )
        .expect("animated scatter render should succeed");

    let meta = std::fs::metadata(&out).expect("output MP4 must exist");
    assert!(
        meta.len() > 1000,
        "output file too small: {} bytes",
        meta.len()
    );
    std::fs::remove_file(&out).ok();
}
