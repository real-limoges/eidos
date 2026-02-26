// src/primitives/line.rs
use crate::Color;
use keyframe_derive::CanTween;

/// A line segment from (x1, y1) to (x2, y2). No fill — only stroke color and width.
#[derive(Debug, Clone)]
pub struct Line {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub stroke_color: Color,
    pub stroke_width: f64,
    pub opacity: f64,
}

impl Line {
    /// Create a line from (x1, y1) to (x2, y2).
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Line {
            x1,
            y1,
            x2,
            y2,
            stroke_color: Color::WHITE,
            stroke_width: 1.0,
            opacity: 1.0,
        }
    }

    /// Set the stroke color. Returns Self for chaining (no validation needed).
    pub fn stroke_color(mut self, color: Color) -> Self {
        self.stroke_color = color;
        self
    }

    /// Set the stroke width. Negative widths are clamped to 0.0.
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width.max(0.0);
        self
    }

    /// Set opacity in [0.0, 1.0]. Values outside [0.0, 1.0] are clamped.
    pub fn opacity(mut self, value: f64) -> Self {
        self.opacity = value.clamp(0.0, 1.0);
        self
    }

    /// Convert to an svg::node::element::Line node for inclusion in an SVG document.
    pub fn to_svg_element(&self) -> svg::node::element::Line {
        use svg::node::element::Line as SvgLine;
        SvgLine::new()
            .set("x1", self.x1)
            .set("y1", self.y1)
            .set("x2", self.x2)
            .set("y2", self.y2)
            .set("stroke", self.stroke_color.to_hex())
            .set("stroke-width", self.stroke_width)
            .set("opacity", self.opacity)
    }
}

/// Animatable state for Line. All fields are f64 for CanTween compatibility.
/// Color channels are 0.0..=255.0; stroke_width and opacity are non-negative.
#[derive(Clone, CanTween)]
pub struct LineState {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub stroke_r: f64,
    pub stroke_g: f64,
    pub stroke_b: f64,
    pub stroke_width: f64,
    pub opacity: f64,
}

impl LineState {
    /// Build a Line from this interpolated state.
    /// Color channels are clamped to [0, 255] then cast to u8.
    pub fn to_line(&self) -> Line {
        let r = self.stroke_r.clamp(0.0, 255.0) as u8;
        let g = self.stroke_g.clamp(0.0, 255.0) as u8;
        let b = self.stroke_b.clamp(0.0, 255.0) as u8;
        Line::new(self.x1, self.y1, self.x2, self.y2)
            .stroke_color(crate::Color::rgb(r, g, b))
            .stroke_width(self.stroke_width.max(0.0))
            .opacity(self.opacity.clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn line_negative_stroke_is_clamped() {
        let result = Line::new(0.0, 0.0, 100.0, 100.0).stroke_width(-1.0);
        assert_eq!(result.stroke_width, 0.0);
    }

    #[test]
    fn line_opacity_clamped() {
        let high = Line::new(0.0, 0.0, 100.0, 100.0).opacity(1.5);
        assert_eq!(high.opacity, 1.0);
        let low = Line::new(0.0, 0.0, 100.0, 100.0).opacity(-0.5);
        assert_eq!(low.opacity, 0.0);
    }

    #[test]
    fn line_valid_chain_succeeds() {
        let l = Line::new(0.0, 0.0, 100.0, 100.0)
            .stroke_color(Color::RED)
            .stroke_width(2.0)
            .opacity(0.8);
        assert_eq!(l.x1, 0.0);
        assert_eq!(l.x2, 100.0);
        assert_eq!(l.stroke_color, Color::RED);
        assert_eq!(l.stroke_width, 2.0);
    }
}
