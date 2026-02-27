// src/primitives/bezier.rs
use crate::Color;

/// An individual path drawing command.
pub enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    /// Cubic bezier: (cx1, cy1) and (cx2, cy2) are control points; (x, y) is the endpoint.
    CubicTo(f64, f64, f64, f64, f64, f64),
    Close,
}

/// A general bezier path primitive.
///
/// Supports move_to, line_to, cubic_to (cubic bezier), and close commands.
/// All path-building methods return Self (infallible). stroke() and opacity() clamp invalid inputs.
///
/// Named `Bezier` to match the existing Primitive enum variant and module export.
#[derive(Debug, Clone)]
pub struct Bezier {
    pub commands: Vec<PathCommand>,
    pub stroke: Option<(Color, f64)>,
    pub fill: Option<Color>,
    pub opacity: f64,
}

// Manual Debug/Clone for PathCommand since f64 doesn't need special handling.
impl std::fmt::Debug for PathCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathCommand::MoveTo(x, y) => write!(f, "MoveTo({}, {})", x, y),
            PathCommand::LineTo(x, y) => write!(f, "LineTo({}, {})", x, y),
            PathCommand::CubicTo(cx1, cy1, cx2, cy2, x, y) => {
                write!(
                    f,
                    "CubicTo({}, {}, {}, {}, {}, {})",
                    cx1, cy1, cx2, cy2, x, y
                )
            }
            PathCommand::Close => write!(f, "Close"),
        }
    }
}

impl Clone for PathCommand {
    fn clone(&self) -> Self {
        match self {
            PathCommand::MoveTo(x, y) => PathCommand::MoveTo(*x, *y),
            PathCommand::LineTo(x, y) => PathCommand::LineTo(*x, *y),
            PathCommand::CubicTo(cx1, cy1, cx2, cy2, x, y) => {
                PathCommand::CubicTo(*cx1, *cy1, *cx2, *cy2, *x, *y)
            }
            PathCommand::Close => PathCommand::Close,
        }
    }
}

impl Bezier {
    /// Create an empty bezier path.
    pub fn new() -> Self {
        Bezier {
            commands: Vec::new(),
            stroke: None,
            fill: None,
            opacity: 1.0,
        }
    }

    /// Move the current point to (x, y) without drawing. Returns Self for chaining.
    pub fn move_to(mut self, x: f64, y: f64) -> Self {
        self.commands.push(PathCommand::MoveTo(x, y));
        self
    }

    /// Draw a straight line to (x, y). Returns Self for chaining.
    pub fn line_to(mut self, x: f64, y: f64) -> Self {
        self.commands.push(PathCommand::LineTo(x, y));
        self
    }

    /// Draw a cubic bezier to (x, y) using (cx1, cy1) and (cx2, cy2) as control points.
    /// Returns Self for chaining.
    pub fn cubic_to(mut self, cx1: f64, cy1: f64, cx2: f64, cy2: f64, x: f64, y: f64) -> Self {
        self.commands
            .push(PathCommand::CubicTo(cx1, cy1, cx2, cy2, x, y));
        self
    }

    /// Close the current subpath (draws line back to the last MoveTo point).
    /// Returns Self for chaining.
    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }

    /// Set stroke color and width. Negative widths are clamped to 0.0.
    pub fn stroke(mut self, color: Color, width: f64) -> Self {
        self.stroke = Some((color, width.max(0.0)));
        self
    }

    /// Set fill color. Returns Self for chaining (no validation needed).
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Set opacity in [0.0, 1.0]. Values outside [0.0, 1.0] are clamped.
    pub fn opacity(mut self, value: f64) -> Self {
        self.opacity = value.clamp(0.0, 1.0);
        self
    }

    /// Convert to an svg::node::element::Path node for inclusion in an SVG document.
    pub fn to_svg_element(&self) -> svg::node::element::Path {
        use svg::node::element::path::Data;
        use svg::node::element::Path as SvgPath;

        let mut data = Data::new();
        for cmd in &self.commands {
            data = match cmd {
                PathCommand::MoveTo(x, y) => data.move_to((*x, *y)),
                PathCommand::LineTo(x, y) => data.line_to((*x, *y)),
                PathCommand::CubicTo(cx1, cy1, cx2, cy2, x, y) => {
                    data.cubic_curve_to((*cx1, *cy1, *cx2, *cy2, *x, *y))
                }
                PathCommand::Close => data.close(),
            };
        }

        let mut el = SvgPath::new().set("d", data).set("opacity", self.opacity);

        el = match self.fill {
            Some(c) => el.set("fill", c.to_hex()),
            None => el.set("fill", "none"),
        };

        if let Some((color, width)) = self.stroke {
            el = el.set("stroke", color.to_hex()).set("stroke-width", width);
        }

        el
    }
}

impl Default for Bezier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn bezier_negative_stroke_is_clamped() {
        let result = Bezier::new().stroke(Color::WHITE, -1.0);
        assert_eq!(result.stroke, Some((Color::WHITE, 0.0)));
    }

    #[test]
    fn bezier_opacity_clamped() {
        let result = Bezier::new().opacity(2.0);
        assert_eq!(result.opacity, 1.0);
    }

    #[test]
    fn bezier_path_building_returns_self() {
        let path = Bezier::new()
            .move_to(0.0, 0.0)
            .line_to(50.0, 50.0)
            .cubic_to(60.0, 0.0, 100.0, 0.0, 100.0, 50.0)
            .close();
        assert_eq!(path.commands.len(), 4);
    }

    #[test]
    fn bezier_valid_chain_with_stroke_and_fill() {
        let path = Bezier::new()
            .move_to(0.0, 0.0)
            .line_to(100.0, 0.0)
            .fill(Color::BLUE)
            .stroke(Color::WHITE, 2.0)
            .opacity(0.75);
        assert!(path.fill.is_some());
        assert!(path.stroke.is_some());
        assert_eq!(path.opacity, 0.75);
    }
}
