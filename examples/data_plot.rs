// examples/data_plot.rs
//
// Renders a 2D data plot with:
//   - Cartesian axes with auto-range, ticks, labels, and grid lines (DATA-01, DATA-03)
//   - A smooth Catmull-Rom cubic spline through data points (DATA-02)
//   - A second curve to demonstrate multi-curve composition
//   - X and Y axis titles
//
// Output: /tmp/data_plot.mp4

use eidos::{Axes, Color, DataCurve, Scene};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sine wave: 20 points over [0, 2pi]
    let sine_data: Vec<(f64, f64)> = (0..=20)
        .map(|i| {
            let x = i as f64 * std::f64::consts::TAU / 20.0;
            (x, x.sin())
        })
        .collect();

    // Cosine wave: 20 points over [0, 2pi]
    let cosine_data: Vec<(f64, f64)> = (0..=20)
        .map(|i| {
            let x = i as f64 * std::f64::consts::TAU / 20.0;
            (x, x.cos())
        })
        .collect();

    let sine_curve = DataCurve::new(sine_data)?
        .stroke(Color::CYAN, 2.5)?;

    let cosine_curve = DataCurve::new(cosine_data)?
        .stroke(Color::rgb(255, 165, 0), 2.5)?;  // orange

    // Axes auto-range -- both curves fit within [-1.1, 1.1] y with 7% padding
    let axes = Axes::new(80.0, 60.0, 840.0, 480.0)
        .x_title("x (radians)")
        .y_title("y")
        .add_curve(sine_curve)
        .add_curve(cosine_curve);

    let scene = Scene::new(1024, 640, 24)?.duration(1.0);

    scene.render_static(|s| {
        s.add_axes(&axes);
    }, "/tmp/data_plot.mp4")?;

    println!("Rendered: /tmp/data_plot.mp4");
    Ok(())
}
