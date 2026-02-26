pub mod axes;
pub mod colormap;
pub mod camera;
pub mod confidence_band;
pub mod data_curve;
pub(crate) mod spline;
pub mod spline_fit;
pub mod surface_plot;

pub use axes::{Axes, AxisRange};
pub use camera::{Camera, Point2D, Point3D, Vector3D};
pub use confidence_band::ConfidenceBand;
pub use data_curve::DataCurve;
pub use spline_fit::SplineFit;
pub use surface_plot::SurfacePlot;
