// tests/gam_viz.rs

use eidos::{Axes, Color, ConfidenceBand, DataCurve, Easing, Scene, SplineFit};
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

#[test]
fn confidence_band_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    let upper = vec![(0.0, 1.3), (5.0, 0.5), (10.0, 1.1)];
    let lower = vec![(0.0, 0.7), (5.0, -0.1), (10.0, 0.5)];
    let band = ConfidenceBand::new(upper.clone(), lower.clone())
        .expect("valid band")
        .fill_color(Color::rgb(100, 149, 237))
        .opacity(0.25);

    let fitted = vec![(0.0, 1.0), (5.0, 0.2), (10.0, 0.8)];
    let curve = DataCurve::new(fitted).expect("valid curve");

    let axes = Axes::new(80.0, 60.0, 800.0, 500.0)
        .x_range(0.0, 10.0)
        .y_range(-0.5, 1.5)
        .add_band(band)
        .add_curve(curve);

    let out = temp_mp4("gam_band_test.mp4");
    let scene = Scene::new(1024, 640, 24).expect("valid scene");
    scene
        .render_static(|s| { s.add_axes(&axes); }, &out)
        .expect("render should succeed");

    let meta = std::fs::metadata(&out).expect("output file must exist");
    assert!(meta.len() > 1000, "output file too small: {} bytes", meta.len());
    std::fs::remove_file(&out).ok();
}

#[test]
fn spline_fit_animation_renders_to_mp4() {
    if !ffmpeg_available() {
        return;
    }

    let pts: Vec<(f64, f64)> = (0..=10)
        .map(|i| (i as f64, (i as f64 * 0.5).sin()))
        .collect();
    let spline = SplineFit::new(pts.clone())
        .expect("valid spline")
        .color(Color::rgb(255, 200, 50))
        .stroke_width(2.5)
        .animate_fit(0.0, 2.0, Easing::EaseOut);

    let axes = Axes::new(80.0, 60.0, 800.0, 500.0)
        .x_range(0.0, 10.0)
        .y_range(-1.5, 1.5);

    let (x_min, x_max, y_min, y_max) = axes.plot_bounds();
    let visual_pts: Vec<(f64, f64)> = pts
        .iter()
        .map(|&(x, y)| {
            let px = axes.x + (x - x_min) / (x_max - x_min) * axes.width;
            let py = (axes.y + axes.height) - (y - y_min) / (y_max - y_min) * axes.height;
            (px, py)
        })
        .collect();

    let out = temp_mp4("gam_spline_test.mp4");
    let scene = Scene::new(1024, 640, 24).expect("valid scene").duration(3.0);

    scene
        .render(
            |s, t_secs| {
                s.add_axes(&axes);
                if let Some(bez) = spline.to_bezier(&visual_pts, t_secs) {
                    s.add(bez);
                }
            },
            &out,
        )
        .expect("render should succeed");

    let meta = std::fs::metadata(&out).expect("output file must exist");
    assert!(meta.len() > 1000, "output file too small: {} bytes", meta.len());
    std::fs::remove_file(&out).ok();
}
