// src/primitives/rect.rs
use crate::{Color, EidosError};

/// A rectangle primitive with top-left origin.
#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f64,      // top-left x
    pub y: f64,      // top-left y
    pub width: f64,
    pub height: f64,
    pub fill: Option<Color>,
    pub stroke: Option<(Color, f64)>, // (color, width)
    pub opacity: f64,
}

impl Rect {
    /// Create a rectangle with top-left corner at (x, y) with given width and height.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rect {
            x,
            y,
            width,
            height,
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

    /// Convert to an svg::node::element::Rectangle node for inclusion in an SVG document.
    pub fn to_svg_element(&self) -> svg::node::element::Rectangle {
        use svg::node::element::Rectangle;

        let mut el = Rectangle::new()
            .set("x", self.x)
            .set("y", self.y)
            .set("width", self.width)
            .set("height", self.height)
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
    fn rect_negative_stroke_returns_err() {
        let result = Rect::new(0.0, 0.0, 100.0, 50.0).stroke(Color::WHITE, -1.0);
        assert!(result.is_err());
    }

    #[test]
    fn rect_opacity_out_of_range_returns_err() {
        let result = Rect::new(0.0, 0.0, 100.0, 50.0).opacity(1.5);
        assert!(result.is_err());
    }

    #[test]
    fn rect_valid_chain_succeeds() {
        let rect = Rect::new(10.0, 20.0, 200.0, 100.0)
            .fill(Color::BLUE)
            .stroke(Color::WHITE, 2.0)
            .unwrap()
            .opacity(0.8)
            .unwrap();
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.fill, Some(Color::BLUE));
    }
}
