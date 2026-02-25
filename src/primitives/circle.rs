// src/primitives/circle.rs
use crate::{Color, EidosError};

/// A circle primitive defined by its center and radius.
#[derive(Debug, Clone)]
pub struct Circle {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub fill: Option<Color>,
    pub stroke: Option<(Color, f64)>, // (color, width)
    pub opacity: f64,
}

impl Circle {
    /// Create a circle centered at (cx, cy) with radius r.
    pub fn new(cx: f64, cy: f64, r: f64) -> Self {
        Circle {
            cx,
            cy,
            r,
            fill: None,
            stroke: None,
            opacity: 1.0,
        }
    }

    /// Set fill color. Returns Self for chaining (fill accepts any Color, no validation needed).
    pub fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Set stroke color and width. Returns Err if width is negative.
    pub fn stroke(mut self, color: Color, width: f64) -> Result<Self, EidosError> {
        if width < 0.0 {
            return Err(EidosError::InvalidConfig(
                "stroke width must be non-negative".into(),
            ));
        }
        self.stroke = Some((color, width));
        Ok(self)
    }

    /// Set opacity in [0.0, 1.0]. Returns Err if outside range.
    pub fn opacity(mut self, value: f64) -> Result<Self, EidosError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(EidosError::InvalidConfig(
                "opacity must be in range [0.0, 1.0]".into(),
            ));
        }
        self.opacity = value;
        Ok(self)
    }

    /// Convert to an svg::node::element::Circle node for inclusion in an SVG document.
    pub fn to_svg_element(&self) -> svg::node::element::Circle {
        use svg::node::element::Circle as SvgCircle;

        let mut el = SvgCircle::new()
            .set("cx", self.cx)
            .set("cy", self.cy)
            .set("r", self.r)
            .set("opacity", self.opacity);

        el = match self.fill {
            Some(c) => el.set("fill", c.to_hex()),
            None => el.set("fill", "none"),
        };

        if let Some((color, width)) = self.stroke {
            el = el
                .set("stroke", color.to_hex())
                .set("stroke-width", width);
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn circle_negative_stroke_returns_err() {
        let result = Circle::new(50.0, 50.0, 30.0).stroke(Color::WHITE, -0.1);
        assert!(result.is_err());
    }

    #[test]
    fn circle_opacity_out_of_range_returns_err() {
        let result = Circle::new(50.0, 50.0, 30.0).opacity(2.0);
        assert!(result.is_err());
    }

    #[test]
    fn circle_valid_chain_succeeds() {
        let c = Circle::new(100.0, 100.0, 50.0)
            .fill(Color::RED)
            .stroke(Color::WHITE, 2.0)
            .unwrap()
            .opacity(0.9)
            .unwrap();
        assert_eq!(c.cx, 100.0);
        assert_eq!(c.fill, Some(Color::RED));
    }
}
