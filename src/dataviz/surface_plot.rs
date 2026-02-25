use crate::dataviz::camera::Point3D;

pub struct SurfacePlot { /* fields added in Task 2 */ }

impl SurfacePlot {
    pub fn new(_xs: Vec<f64>, _ys: Vec<f64>, _zs: Vec<f64>, _rows: usize, _cols: usize) -> Self {
        todo!()
    }
    pub fn world_point(&self, _row: usize, _col: usize) -> Point3D {
        todo!()
    }
    pub fn rows(&self) -> usize { todo!() }
    pub fn cols(&self) -> usize { todo!() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_panics_on_xs_length_mismatch() {
        let result = std::panic::catch_unwind(|| {
            SurfacePlot::new(vec![1.0, 2.0], vec![1.0, 2.0], vec![1.0], 1, 2);
        });
        assert!(result.is_err(), "should panic when zs.len() != rows * cols");
    }

    #[test]
    fn new_panics_on_ys_length_mismatch() {
        let result = std::panic::catch_unwind(|| {
            SurfacePlot::new(vec![1.0, 2.0], vec![1.0], vec![1.0, 2.0], 1, 2);
        });
        assert!(result.is_err(), "should panic when ys.len() != rows * cols");
    }

    #[test]
    fn normalizes_x_min_to_neg1_and_max_to_pos1() {
        // 1x3 grid: x in [0, 10], y constant, z constant
        let xs = vec![0.0, 5.0, 10.0];
        let ys = vec![0.0, 0.0, 0.0];
        let zs = vec![0.0, 0.0, 0.0];
        let plot = SurfacePlot::new(xs, ys, zs, 1, 3);
        let p_min = plot.world_point(0, 0);
        let p_max = plot.world_point(0, 2);
        assert!((p_min.x - (-1.0)).abs() < 1e-10, "x_min should normalize to -1; got {}", p_min.x);
        assert!((p_max.x - 1.0).abs() < 1e-10, "x_max should normalize to +1; got {}", p_max.x);
    }

    #[test]
    fn normalizes_z_axis_independently() {
        // z in [2, 8] should normalize to [-1, 1]
        let xs = vec![0.0, 1.0];
        let ys = vec![0.0, 0.0];
        let zs = vec![2.0, 8.0];
        let plot = SurfacePlot::new(xs, ys, zs, 1, 2);
        let p0 = plot.world_point(0, 0);
        let p1 = plot.world_point(0, 1);
        assert!((p0.z - (-1.0)).abs() < 1e-10, "z_min should normalize to -1; got {}", p0.z);
        assert!((p1.z - 1.0).abs() < 1e-10, "z_max should normalize to +1; got {}", p1.z);
    }

    #[test]
    fn flat_surface_no_panic_z_maps_to_zero() {
        // All z values equal → degenerate range → should NOT divide by zero; all z map to 0.0
        let xs = vec![0.0, 1.0, 0.0, 1.0];
        let ys = vec![0.0, 0.0, 1.0, 1.0];
        let zs = vec![5.0, 5.0, 5.0, 5.0];
        let plot = SurfacePlot::new(xs, ys, zs, 2, 2);
        for r in 0..2 {
            for c in 0..2 {
                let p = plot.world_point(r, c);
                assert!((p.z - 0.0).abs() < 1e-10,
                    "flat surface z should normalize to 0.0; got {} at ({},{})", p.z, r, c);
            }
        }
    }

    #[test]
    fn world_point_row_major_ordering() {
        // 2x2 grid: xs[r*cols + c] maps to point (r, c)
        // xs = [0, 10, 0, 10], ys = [0, 0, 10, 10], zs all 0
        // After normalization: x in {-1, +1}, y in {-1, +1}
        let xs = vec![0.0, 10.0, 0.0, 10.0];
        let ys = vec![0.0, 0.0, 10.0, 10.0];
        let zs = vec![0.0; 4];
        let plot = SurfacePlot::new(xs, ys, zs, 2, 2);
        let p00 = plot.world_point(0, 0); // xs[0]=0 → x=-1, ys[0]=0 → y=-1
        let p01 = plot.world_point(0, 1); // xs[1]=10 → x=+1
        let p10 = plot.world_point(1, 0); // ys[2]=10 → y=+1
        assert!((p00.x - (-1.0)).abs() < 1e-10);
        assert!((p01.x - 1.0).abs() < 1e-10);
        assert!((p00.y - (-1.0)).abs() < 1e-10);
        assert!((p10.y - 1.0).abs() < 1e-10);
    }

    #[test]
    fn rows_and_cols_accessors() {
        let plot = SurfacePlot::new(vec![0.0; 6], vec![0.0; 6], vec![0.0; 6], 2, 3);
        assert_eq!(plot.rows(), 2);
        assert_eq!(plot.cols(), 3);
    }
}
