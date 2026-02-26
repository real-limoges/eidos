// examples/gam_plot.rs
//
// GAM visualization example — demonstrates Phase 4 GAM primitives.
// Run with: cargo run --example gam_plot
// Output: /tmp/gam_plot.mp4
//
// Shows:
//   - ConfidenceBand: semi-transparent shaded region between upper/lower sin(x) ± 0.3 (GAM-01)
//   - DataCurve: white fitted line through sin(x) over [0, 10]
//   - SplineFit: animated left-to-right Catmull-Rom reveal from t=0.5s to t=3.5s (GAM-02)
//   - Axes: explicit x_range [0, 10] and y_range [-1.4, 1.4] with titles

use eidos::{Axes, Color, ConfidenceBand, DataCurve, Easing, Scene, SplineFit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- Mock GAM data: sine curve as fitted line, ±0.3 as band bounds ---
    let x_vals: Vec<f64> = (0..=20).map(|i| i as f64 * 0.5).collect(); // 0.0 to 10.0
    let fitted: Vec<(f64, f64)> = x_vals.iter().map(|&x| (x, x.sin())).collect();
    let upper: Vec<(f64, f64)> = x_vals.iter().map(|&x| (x, x.sin() + 0.3)).collect();
    let lower: Vec<(f64, f64)> = x_vals.iter().map(|&x| (x, x.sin() - 0.3)).collect();

    // --- ConfidenceBand: cornflower blue fill, 25% opacity ---
    let band = ConfidenceBand::new(upper.clone(), lower.clone())?
        .fill_color(Color::rgb(100, 149, 237))
        .opacity(0.25);

    // --- DataCurve: white fitted line, width 2.0 ---
    let data_curve = DataCurve::new(fitted.clone())?.stroke(Color::WHITE, 2.0);

    // --- Axes with explicit ranges and titles ---
    let axes = Axes::new(80.0, 60.0, 1100.0, 580.0)
        .x_range(0.0, 10.0)
        .y_range(-1.4, 1.4)
        .x_title("x")
        .y_title("sin(x)")
        .add_band(band)
        .add_curve(data_curve);

    // --- SplineFit: animated from t=0.5 to t=3.5 with EaseInOut ---
    let spline = SplineFit::new(fitted.clone())?
        .color(Color::rgb(255, 200, 50))
        .stroke_width(2.5)
        .animate_fit(0.5, 3.0, Easing::EaseInOut);

    // Map fitted points to visual space using Axes::plot_bounds() -- guaranteed to match
    // to_primitives() coordinate mapping (tick-adjusted bounds, same formula).
    let (x_min, x_max, y_min, y_max) = axes.plot_bounds();
    let visual_pts: Vec<(f64, f64)> = fitted
        .iter()
        .map(|&(x, y)| {
            let px = axes.x + (x - x_min) / (x_max - x_min) * axes.width;
            let py = (axes.y + axes.height) - (y - y_min) / (y_max - y_min) * axes.height;
            (px, py)
        })
        .collect();

    // --- Scene: 1280x720, 30fps, 4 seconds ---
    let scene = Scene::new(1280, 720, 30)?.duration(4.0);

    scene.render(
        |s, t_secs| {
            s.add_axes(&axes);
            if let Some(bez) = spline.to_bezier(&visual_pts, t_secs) {
                s.add(bez);
            }
        },
        "/tmp/gam_plot.mp4",
    )?;

    println!("Rendered: /tmp/gam_plot.mp4");
    Ok(())
}
