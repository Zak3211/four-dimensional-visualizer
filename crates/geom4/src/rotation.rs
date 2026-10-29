//! Rotations in 4D.
//!
//! A 4D rotation turns a *plane*, not an axis: the rotation in the XW plane
//! mixes x and w and leaves y and z alone. There are six coordinate planes.

use crate::vec::Vec4;
use std::ops::Mul;

/// One of the six coordinate planes of 4D space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Plane {
    XY,
    XZ,
    XW,
    YZ,
    YW,
    ZW,
}

impl Plane {
    /// All six planes, in the order [`Mat4::from_angles`] composes them.
    pub const ALL: [Plane; 6] = [
        Plane::XY,
        Plane::XZ,
        Plane::XW,
        Plane::YZ,
        Plane::YW,
        Plane::ZW,
    ];

    /// Coordinate indices `(i, j)` spanning the plane, with `i < j` (x=0 … w=3).
    pub const fn axes(self) -> (usize, usize) {
        match self {
            Plane::XY => (0, 1),
            Plane::XZ => (0, 2),
            Plane::XW => (0, 3),
            Plane::YZ => (1, 2),
            Plane::YW => (1, 3),
            Plane::ZW => (2, 3),
        }
    }

    /// Short label such as `"XW"`.
    pub const fn label(self) -> &'static str {
        match self {
            Plane::XY => "XY",
            Plane::XZ => "XZ",
            Plane::XW => "XW",
            Plane::YZ => "YZ",
            Plane::YW => "YW",
            Plane::ZW => "ZW",
        }
    }
}

/// A 4×4 matrix acting on column vectors: `(m * v)[r] = Σ_c m.0[r][c] * v[c]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4(pub [[f64; 4]; 4]);

impl Mat4 {
    pub const fn identity() -> Mat4 {
        Mat4([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Rotation by `angle` radians in `plane`, turning the plane's first axis
    /// towards its second (XY by +π/2 sends x to y).
    pub fn rotation(plane: Plane, angle: f64) -> Mat4 {
        let (i, j) = plane.axes();
        let (s, c) = angle.sin_cos();
        let mut m = Mat4::identity();
        m.0[i][i] = c;
        m.0[j][j] = c;
        m.0[j][i] = s;
        m.0[i][j] = -s;
        m
    }

    /// Composes one rotation per plane. `angles[k]` belongs to `Plane::ALL[k]`;
    /// the XY rotation is applied first and the ZW rotation last, i.e.
    /// `R = R_ZW · R_YW · … · R_XY`.
    pub fn from_angles(angles: &[f64; 6]) -> Mat4 {
        Plane::ALL
            .iter()
            .zip(angles)
            .fold(Mat4::identity(), |acc, (&plane, &angle)| {
                Mat4::rotation(plane, angle) * acc
            })
    }
}

impl Mul<Mat4> for Mat4 {
    type Output = Mat4;

    /// Composition: `(a * b) * v == a * (b * v)`.
    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut out = [[0.0; 4]; 4];
        for (r, row) in out.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                *cell = (0..4).map(|k| self.0[r][k] * rhs.0[k][c]).sum();
            }
        }
        Mat4(out)
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, v: Vec4) -> Vec4 {
        let v = v.to_array();
        let row = |r: usize| (0..4).map(|c| self.0[r][c] * v[c]).sum();
        Vec4::new(row(0), row(1), row(2), row(3))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, TAU};

    const EPS: f64 = 1e-12;

    fn assert_mat_close(a: &Mat4, b: &Mat4) {
        for r in 0..4 {
            for c in 0..4 {
                assert!(
                    (a.0[r][c] - b.0[r][c]).abs() < EPS,
                    "entry ({r},{c}): {} vs {}\n{a:?}\n{b:?}",
                    a.0[r][c],
                    b.0[r][c]
                );
            }
        }
    }

    fn transpose(m: &Mat4) -> Mat4 {
        let mut t = [[0.0; 4]; 4];
        for (r, row) in t.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                *cell = m.0[c][r];
            }
        }
        Mat4(t)
    }

    /// Determinant by Gaussian elimination with partial pivoting.
    fn det(m: &Mat4) -> f64 {
        let mut a = m.0;
        let mut d = 1.0;
        for col in 0..4 {
            let p = (col..4)
                .max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))
                .unwrap();
            if a[p][col] == 0.0 {
                return 0.0;
            }
            if p != col {
                a.swap(p, col);
                d = -d;
            }
            d *= a[col][col];
            for r in col + 1..4 {
                let f = a[r][col] / a[col][col];
                let pivot = a[col];
                for (x, p) in a[r].iter_mut().zip(pivot).skip(col) {
                    *x -= f * p;
                }
            }
        }
        d
    }

    fn sample_angle_sets() -> Vec<[f64; 6]> {
        vec![
            [0.3, -1.2, 2.5, 0.7, -0.4, 1.9],
            [TAU, 1.0, -3.0, 0.0, 5.5, 0.1],
            [FRAC_PI_2; 6],
            [-0.01, 0.02, -0.03, 0.04, -0.05, 0.06],
        ]
    }

    #[test]
    fn every_rotation_is_orthogonal_with_det_one() {
        let mut mats: Vec<Mat4> = sample_angle_sets().iter().map(Mat4::from_angles).collect();
        for plane in Plane::ALL {
            mats.push(Mat4::rotation(plane, 0.83));
        }
        for m in &mats {
            assert_mat_close(&(*m * transpose(m)), &Mat4::identity());
            assert!((det(m) - 1.0).abs() < EPS, "det = {}", det(m));
        }
    }

    #[test]
    fn xw_rotation_leaves_y_and_z_unchanged() {
        let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let r = Mat4::rotation(Plane::XW, 0.7) * v;
        assert_eq!((r.y, r.z), (2.0, 3.0));
        assert!((r.norm() - v.norm()).abs() < EPS);
    }

    #[test]
    fn rotation_direction_first_axis_towards_second() {
        let r = Mat4::rotation(Plane::XY, FRAC_PI_2) * Vec4::new(1.0, 0.0, 0.0, 0.0);
        assert!((r - Vec4::new(0.0, 1.0, 0.0, 0.0)).norm() < EPS);
    }

    #[test]
    fn full_turn_and_zero_angles_give_identity() {
        for plane in Plane::ALL {
            assert_mat_close(&Mat4::rotation(plane, TAU), &Mat4::identity());
        }
        assert_mat_close(&Mat4::from_angles(&[TAU; 6]), &Mat4::identity());
        assert_eq!(Mat4::from_angles(&[0.0; 6]), Mat4::identity());
    }

    #[test]
    fn from_angles_composes_in_plane_order() {
        let a = [0.3, -1.2, 2.5, 0.7, -0.4, 1.9];
        let expected = Plane::ALL
            .iter()
            .zip(a)
            .rev()
            .map(|(&p, t)| Mat4::rotation(p, t))
            .fold(Mat4::identity(), |acc, m| acc * m);
        assert_mat_close(&Mat4::from_angles(&a), &expected);
    }

    #[test]
    fn matrix_vector_product_is_associative_with_composition() {
        let a = Mat4::from_angles(&[0.1, 0.2, 0.3, 0.4, 0.5, 0.6]);
        let b = Mat4::rotation(Plane::YW, -1.1);
        let v = Vec4::new(1.0, -2.0, 0.5, 3.0);
        assert!(((a * b) * v - a * (b * v)).norm() < EPS);
    }
}
