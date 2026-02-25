// src/primitives/mod.rs
// Primitive types implemented in plans 01-03 and 01-04
pub mod circle;
pub mod rect;
pub mod line;
pub mod arrow;
pub mod text;
pub mod bezier;

/// Enum over all primitive types for dispatch in the SVG pipeline.
/// Individual struct fields are implemented in plans 01-03 (circle, rect, line, text)
/// and 01-04 (arrow, bezier). The SVG conversion dispatch is completed in plan 01-05.
#[derive(Debug, Clone)]
pub enum Primitive {
    Circle(circle::Circle),
    Rect(rect::Rect),
    Line(line::Line),
    Arrow(arrow::Arrow),
    Text(text::Text),
    Bezier(bezier::Bezier),
}
