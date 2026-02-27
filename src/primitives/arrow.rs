// src/primitives/arrow.rs
use crate::Color;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global counter for generating unique arrow marker IDs.
/// Unique IDs are required to avoid SVG ID conflicts when multiple arrows appear in one scene.
static ARROW_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_arrow_id() -> u64 {
    ARROW_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// A directed line segment (arrow) from (x1, y1) to (x2, y2).
///
/// Arrow produces two SVG nodes: a `<defs>` block containing the arrowhead marker,
/// and a `<line>` element referencing it via `marker-end`. The defs block MUST be
/// added to the SVG document before the line element (SVG spec: defs precede references).
#[derive(Debug, Clone)]
pub struct Arrow {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub stroke_color: Color,
    pub stroke_width: f64,
    pub opacity: f64,
    id: u64, // unique per instance for SVG marker ID collision avoidance
}

impl Arrow {
    /// Create an arrow from (x1, y1) to (x2, y2). Arrowhead appears at (x2, y2).
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Arrow {
            x1,
            y1,
            x2,
            y2,
            stroke_color: Color::WHITE,
            stroke_width: 1.0,
            opacity: 1.0,
            id: next_arrow_id(),
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

    /// Returns (Definitions, Line) — the SVG defs block with the arrowhead marker and the
    /// line element referencing it.
    ///
    /// IMPORTANT: `svg_gen` MUST add the Definitions to the document BEFORE the Line,
    /// otherwise the `url(#marker-id)` reference in `marker-end` will be unresolved.
    pub fn to_svg_parts(&self) -> (svg::node::element::Definitions, svg::node::element::Line) {
        use svg::node::element::path::Data;
        use svg::node::element::{Definitions, Line as SvgLine, Marker, Path as SvgPath};

        let marker_id = format!("arrow-{}", self.id);
        let color_hex = self.stroke_color.to_hex();

        // Use Path with M/L/Z for the arrowhead polygon — avoids relying on the svg crate
        // exposing a Polygon element type.
        let arrowhead_data = Data::new()
            .move_to((0, 0))
            .line_to((10, 5))
            .line_to((0, 10))
            .close();

        let arrowhead = SvgPath::new()
            .set("d", arrowhead_data)
            .set("fill", color_hex.clone());

        let marker = Marker::new()
            .set("id", marker_id.clone())
            .set("markerWidth", 10)
            .set("markerHeight", 10)
            .set("refX", 9) // position the arrowhead tip at the line endpoint
            .set("refY", 5)
            .set("orient", "auto")
            .add(arrowhead);

        let defs = Definitions::new().add(marker);

        let line = SvgLine::new()
            .set("x1", self.x1)
            .set("y1", self.y1)
            .set("x2", self.x2)
            .set("y2", self.y2)
            .set("stroke", color_hex)
            .set("stroke-width", self.stroke_width)
            .set("opacity", self.opacity)
            .set("marker-end", format!("url(#{})", marker_id));

        (defs, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Color;

    #[test]
    fn arrow_negative_stroke_is_clamped() {
        let result = Arrow::new(0.0, 0.0, 100.0, 100.0).stroke_width(-1.0);
        assert_eq!(result.stroke_width, 0.0);
    }

    #[test]
    fn arrow_opacity_clamped() {
        let result = Arrow::new(0.0, 0.0, 100.0, 100.0).opacity(1.5);
        assert_eq!(result.opacity, 1.0);
    }

    #[test]
    fn arrow_unique_marker_ids() {
        let a1 = Arrow::new(0.0, 0.0, 50.0, 50.0);
        let a2 = Arrow::new(100.0, 100.0, 200.0, 200.0);
        // Each arrow instance gets a unique id from the atomic counter
        assert_ne!(a1.id, a2.id);
    }

    #[test]
    fn arrow_to_svg_parts_returns_defs_and_line() {
        let arrow = Arrow::new(0.0, 0.0, 100.0, 100.0).stroke_color(Color::RED);
        let (_defs, _line) = arrow.to_svg_parts();
        // Compilation of this call confirms the return type is (Definitions, Line)
    }
}
