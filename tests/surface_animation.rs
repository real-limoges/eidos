// tests/surface_animation.rs
//! Integration tests for Phase 7: surface morph + camera orbit animation.

use eidos::{Camera, Easing, Scene, SurfacePlot};
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

/// Build a simple paraboloid surface: z = x^2 + y^2 over a 10x10 grid.
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

/// ANIM-01: Surface morphs from flat (z=0) to its paraboloid shape over 3 seconds.
/// ANIM-02: Camera orbits from azimuth=45° to azimuth=135° over 3 seconds.
/// Both animations run simultaneously (overlapping time ranges is supported).
#[test]
fn surface_animation_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    let plot = make_paraboloid(10)
        .animate_fit(0.0, 3.0, Easing::EaseOut)
        .animate_camera_azimuth(0.0, 3.0, 45.0, 135.0, Easing::Linear);

    let out = temp_mp4("surface_animation_test.mp4");

    let scene = Scene::new(800, 600, 24)
        .expect("valid scene config")
        .duration(3.0);

    // elevation_deg and distance are fixed throughout (only azimuth animates)
    let elevation_deg = 30.0_f64;
    let distance = 3.0_f64;

    scene
        .render(
            |s, t_secs| {
                // Resolve animated azimuth, or fall back to static 45°
                let azimuth = plot.camera_at(t_secs).unwrap_or(45.0);
                let camera = Camera::new(azimuth, elevation_deg, distance);
                s.add_surface_at(&plot, &camera, (800, 600), t_secs);
            },
            &out,
        )
        .expect("surface animation render should succeed");

    let meta = std::fs::metadata(&out).expect("output MP4 must exist");
    assert!(
        meta.len() > 1000,
        "output file too small: {} bytes",
        meta.len()
    );
    std::fs::remove_file(&out).ok();
}

/// ANIM-01 only: surface morphs with a static camera (no animate_camera_azimuth).
/// Verifies that to_primitives_at works standalone without camera animation.
#[test]
fn surface_morph_only_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    let plot = make_paraboloid(8).animate_fit(0.0, 2.0, Easing::EaseInOut);

    let static_camera = Camera::new(60.0, 30.0, 3.0);
    let out = temp_mp4("surface_morph_only_test.mp4");

    let scene = Scene::new(800, 600, 24)
        .expect("valid scene config")
        .duration(2.0);

    scene
        .render(
            |s, t_secs| {
                s.add_surface_at(&plot, &static_camera, (800, 600), t_secs);
            },
            &out,
        )
        .expect("surface morph render should succeed");

    let meta = std::fs::metadata(&out).expect("output MP4 must exist");
    assert!(
        meta.len() > 1000,
        "output file too small: {} bytes",
        meta.len()
    );
    std::fs::remove_file(&out).ok();
}

/// ANIM-02 only: camera orbits around a static surface (no animate_fit).
/// Verifies that camera_at works standalone without surface morph.
#[test]
fn camera_orbit_only_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    // No animate_fit — surface renders fully fitted from frame 0
    let plot = make_paraboloid(8).animate_camera_azimuth(0.0, 2.0, 30.0, 210.0, Easing::Linear);

    let out = temp_mp4("camera_orbit_only_test.mp4");

    let scene = Scene::new(800, 600, 24)
        .expect("valid scene config")
        .duration(2.0);

    scene
        .render(
            |s, t_secs| {
                let azimuth = plot.camera_at(t_secs).unwrap_or(30.0);
                let camera = Camera::new(azimuth, 30.0, 3.0);
                // Use to_primitives (not add_surface_at) to verify static surface renders correctly during orbit
                s.add_surface(&plot, &camera, (800, 600));
            },
            &out,
        )
        .expect("camera orbit render should succeed");

    let meta = std::fs::metadata(&out).expect("output MP4 must exist");
    assert!(
        meta.len() > 1000,
        "output file too small: {} bytes",
        meta.len()
    );
    std::fs::remove_file(&out).ok();
}
