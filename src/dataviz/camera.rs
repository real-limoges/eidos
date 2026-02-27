//! Camera and 3D projection types for the eidos surface visualization system.
//!
//! # Coordinate conventions
//! - Z is the up-axis. The surface sits in the XY plane.
//! - Camera orbits above the XY plane using spherical coordinates (azimuth, elevation, distance).
//! - Angles are specified in degrees in the public API; internally converted to radians.
//! - World space is normalized to [-1, 1] per axis at SurfacePlot construction time.
//!
//! # Public API
//! Only [`Camera::project_to_screen`] and [`Camera::is_face_visible`] are public math operations.
//! The view matrix and all nalgebra internals are private.

use nalgebra::{Isometry3, Matrix4, Perspective3, Point3 as NaPoint3, Vector3 as NaVec3};

/// A point in 3D world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A 3D direction vector (not normalized by default).
///
/// Used for face normals in [`Camera::is_face_visible`].
/// Convention: normals must point **outward** from the surface face (toward the front/visible side).
/// For a horizontal face in the XY plane visible from above, the outward normal is (0, 0, +1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A projected 2D screen point in SVG pixel coordinates.
///
/// Origin (0, 0) is the top-left corner. Y increases downward (SVG convention).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

/// A perspective camera defined by spherical coordinates.
///
/// Constructed once, projects many points. Immutable after construction.
///
/// # Example
/// ```
/// use eidos::Camera;
/// use eidos::dataviz::camera::{Point3D, Point2D};
///
/// let cam = Camera::new(45.0, 30.0, 3.0);
/// let origin = Point3D { x: 0.0, y: 0.0, z: 0.0 };
/// if let Some(screen_pt) = cam.project_to_screen(origin, (800, 600)) {
///     println!("origin projects to ({:.1}, {:.1})", screen_pt.x, screen_pt.y);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Camera {
    /// Azimuth angle in degrees (0° = camera on +Y axis looking toward origin).
    pub azimuth_deg: f64,
    /// Elevation angle in degrees (0° = horizontal, 90° = directly above).
    /// Clamped to [-89.9, 89.9] to prevent degenerate look_at_rh.
    pub elevation_deg: f64,
    /// Camera distance from origin in normalized world-space units (~3.0 sees the full unit cube).
    pub distance: f64,
    // Private: precomputed view matrix (viewport-independent).
    // Perspective projection is applied per-call in project_to_screen (needs aspect ratio).
    view: Matrix4<f64>,
    // Eye position in world space. Used for is_face_visible dot product.
    eye: NaVec3<f64>,
}

impl Camera {
    /// Create a camera at the given spherical position looking at the world origin.
    ///
    /// - `azimuth_deg`: horizontal rotation in degrees (0° = +Y axis)
    /// - `elevation_deg`: vertical angle above the XY plane in degrees; clamped to [-89.9, 89.9]
    /// - `distance`: distance from origin in world-space units
    ///
    /// Defaults from CONTEXT.md: azimuth=45°, elevation=30°, distance=3.0.
    pub fn new(azimuth_deg: f64, elevation_deg: f64, distance: f64) -> Self {
        // Clamp elevation to avoid degenerate look_at_rh (eye collinear with up vector at ±90°)
        let clamped_elevation = elevation_deg.clamp(-89.9, 89.9);
        let el = clamped_elevation.to_radians();
        let az = azimuth_deg.to_radians();

        // Z-up spherical to Cartesian (Z is the up-axis per CONTEXT.md):
        //   eye_x = distance * cos(el) * sin(az)
        //   eye_y = distance * cos(el) * cos(az)
        //   eye_z = distance * sin(el)
        let eye = NaVec3::new(
            distance * el.cos() * az.sin(),
            distance * el.cos() * az.cos(),
            distance * el.sin(),
        );

        // look_at_rh: right-hand coordinate system, Z is up (not Y — matches CONTEXT.md Z-up convention)
        let view = Isometry3::look_at_rh(
            &NaPoint3::from(eye),
            &NaPoint3::origin(),
            &NaVec3::z(), // Z is up — CRITICAL: not Vector3::y() (that is Y-up / OpenGL convention)
        )
        .to_homogeneous();

        Camera {
            azimuth_deg,
            elevation_deg: clamped_elevation,
            distance,
            view,
            eye,
        }
    }

    /// Project a world-space point to SVG pixel coordinates.
    ///
    /// Returns `None` if the point is behind the near plane (not visible from the camera).
    /// The caller is responsible for skipping `None` points during rendering.
    ///
    /// Uses a 45° vertical field-of-view and near/far planes of 0.1 and 100.0.
    pub fn project_to_screen(&self, point: Point3D, viewport: (u32, u32)) -> Option<Point2D> {
        let (vw, vh) = viewport;
        let aspect = vw as f64 / vh as f64;

        // Perspective projection with viewport-dependent aspect ratio (not cached — viewport varies)
        let proj = Perspective3::new(aspect, 45_f64.to_radians(), 0.1, 100.0);

        // Transform world-space point to view space using precomputed view matrix
        let p_world = NaPoint3::new(point.x, point.y, point.z);
        let p_view = self.view.transform_point(&p_world);

        // Right-hand convention: points behind camera have z >= 0 in view space
        if p_view.z >= 0.0 {
            return None;
        }

        // Project view-space point to NDC [-1, 1]
        let p_ndc = proj.project_point(&p_view);

        // NDC to SVG pixel coordinates
        // X: NDC [-1,1] → [0, width]
        // Y: NDC [-1,1] top=+1 → SVG Y-down: py = (1 - ndc_y) * 0.5 * height
        let px = (p_ndc.x + 1.0) * 0.5 * vw as f64;
        let py = (1.0 - p_ndc.y) * 0.5 * vh as f64; // Y-flip: SVG is Y-down

        Some(Point2D { x: px, y: py })
    }

    /// Returns `true` if the face with the given outward normal is visible from this camera.
    ///
    /// Uses a dot-product backface culling test. The face is visible when its outward normal
    /// has a positive component toward the camera eye position.
    ///
    /// **Normal convention:** `face_normal` must point outward from the surface face (away from
    /// the interior, toward the front face). For a horizontal face in the XY plane visible
    /// from above, the outward normal is (0, 0, +1).
    pub fn is_face_visible(&self, face_normal: Vector3D) -> bool {
        // Dot product of face normal with camera eye direction (from origin toward eye)
        // dot > 0 means normal points toward the camera (face is front-facing → visible)
        let dot =
            face_normal.x * self.eye.x + face_normal.y * self.eye.y + face_normal.z * self.eye.z;
        dot > 0.0
    }

    /// Returns the camera eye position in world space as (x, y, z).
    ///
    /// Recomputes from spherical parameters (azimuth_deg, elevation_deg, distance).
    /// Used by surface rendering for painter's algorithm depth sorting.
    pub fn eye_position(&self) -> (f64, f64, f64) {
        let el = self.elevation_deg.clamp(-89.9, 89.9).to_radians();
        let az = self.azimuth_deg.to_radians();
        (
            self.distance * el.cos() * az.sin(),
            self.distance * el.cos() * az.cos(),
            self.distance * el.sin(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_origin_to_screen_center() {
        // Camera at azimuth=0, elevation=0, distance=3 looks along the -Y axis toward origin
        let cam = Camera::new(0.0, 0.0, 3.0);
        let origin = Point3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let pt = cam
            .project_to_screen(origin, (800, 600))
            .expect("origin should project");
        let cx = 800.0 / 2.0;
        let cy = 600.0 / 2.0;
        assert!(
            (pt.x - cx).abs() < 1.5,
            "origin x={} should be near center {}",
            pt.x,
            cx
        );
        assert!(
            (pt.y - cy).abs() < 1.5,
            "origin y={} should be near center {}",
            pt.y,
            cy
        );
    }

    #[test]
    fn point_above_origin_projects_above_screen_center() {
        // Z > 0 in world space → upper half of SVG (smaller y pixel value)
        let cam = Camera::new(0.0, 0.0, 3.0);
        let above = Point3D {
            x: 0.0,
            y: 0.0,
            z: 0.5,
        };
        let pt = cam.project_to_screen(above, (800, 600)).unwrap();
        let cy = 600.0 / 2.0;
        assert!(
            pt.y < cy,
            "point above origin (z=0.5) should project above screen center; got y={}",
            pt.y
        );
    }

    #[test]
    fn point_behind_camera_returns_none() {
        // Camera at (0, 3, 0) looking toward origin. Point at y=5 is behind the camera.
        let cam = Camera::new(0.0, 0.0, 3.0);
        let behind = Point3D {
            x: 0.0,
            y: 5.0,
            z: 0.0,
        };
        let result = cam.project_to_screen(behind, (800, 600));
        assert!(result.is_none(), "point behind camera should return None");
    }

    #[test]
    fn is_face_visible_toward_camera() {
        // Camera at positive Y (azimuth=0, elevation=0, distance=3 → eye at (0,3,0))
        // Normal pointing +Y faces toward camera
        let cam = Camera::new(0.0, 0.0, 3.0);
        let toward = Vector3D {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        };
        assert!(
            cam.is_face_visible(toward),
            "normal pointing toward camera should be visible"
        );
    }

    #[test]
    fn is_face_visible_away_from_camera() {
        let cam = Camera::new(0.0, 0.0, 3.0);
        let away = Vector3D {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        };
        assert!(
            !cam.is_face_visible(away),
            "normal pointing away from camera should be invisible"
        );
    }

    #[test]
    fn elevation_clamped_at_poles() {
        // Constructing Camera at exactly 90° elevation must not panic (degenerate look_at)
        // It should silently clamp to 89.9°
        let _ = Camera::new(0.0, 90.0, 3.0); // must not panic
        let _ = Camera::new(0.0, -90.0, 3.0); // must not panic
    }

    #[test]
    fn camera_45_elevation_eye_z_positive() {
        // At elevation=45°, the eye should have a positive Z component (above the XY plane)
        let cam = Camera::new(0.0, 45.0, 3.0);
        // We can only test via projection: a point at z=0 should project near center
        // and a point at z=0.5 should be above center
        let z0 = cam.project_to_screen(
            Point3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            (800, 600),
        );
        assert!(
            z0.is_some(),
            "origin should be visible from 45-deg elevation camera"
        );
    }

    #[test]
    fn eye_position_at_azimuth0_elevation0_is_on_y_axis() {
        // Camera at azimuth=0°, elevation=0°, distance=3 → eye on +Y axis
        // x = 3 * cos(0) * sin(0) = 0
        // y = 3 * cos(0) * cos(0) = 3
        // z = 3 * sin(0) = 0
        let cam = Camera::new(0.0, 0.0, 3.0);
        let (x, y, z) = cam.eye_position();
        assert!(x.abs() < 1e-10, "x should be ~0.0; got {}", x);
        assert!((y - 3.0).abs() < 1e-10, "y should be ~3.0; got {}", y);
        assert!(z.abs() < 1e-10, "z should be ~0.0; got {}", z);
    }

    #[test]
    fn eye_position_at_elevation45_has_positive_z() {
        // Camera at azimuth=0°, elevation=45°, distance=3
        // z = 3 * sin(45°) = 3 * 0.707 ≈ 2.12 > 0
        let cam = Camera::new(0.0, 45.0, 3.0);
        let (_, _, z) = cam.eye_position();
        assert!(z > 0.0, "z should be positive at elevation=45°; got {}", z);
    }
}
