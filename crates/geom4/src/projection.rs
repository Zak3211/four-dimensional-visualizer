//! Projection from 4D down to the 3D "screen".

use crate::vec::{Vec3, Vec4};

/// Below this distance from the 4D eye a point is treated as at the eye.
const EYE_EPS: f64 = 1e-9;

/// How a 4D point is flattened into 3D.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection4 {
    /// A 4D eye at `(0, 0, 0, eye_w)` looking down −w onto the `w = 0` hyperplane:
    /// `xyz * eye_w / (eye_w - w)`. Points with `w ≥ eye_w` are at or behind the eye.
    Perspective { eye_w: f64 },
    /// Drop w.
    Orthographic,
}

impl Default for Projection4 {
    fn default() -> Self {
        Projection4::Perspective { eye_w: 4.0 }
    }
}

impl Projection4 {
    /// `None` when the point is at or behind the 4D eye. A perspective eye must
    /// sit at positive w; otherwise the image is mirrored.
    pub fn project(&self, p: Vec4) -> Option<Vec3> {
        let xyz = Vec3::new(p.x, p.y, p.z);
        match *self {
            Projection4::Orthographic => Some(xyz),
            Projection4::Perspective { eye_w } => {
                debug_assert!(eye_w > 0.0, "4D eye must be at positive w, got {eye_w}");
                let d = eye_w - p.w;
                (d > EYE_EPS).then(|| xyz * (eye_w / d))
            }
        }
    }

    /// The largest `|project(p)|` over all `p` with `|p| <= radius`, so a
    /// camera further away than this is outside the projected shape.
    ///
    /// Perspective: maximising `sqrt(r² - w²) · e / (e - w)` gives `w = r²/e`
    /// and the bound `r·e / sqrt(e² - r²)`, infinite once `e <= r`.
    pub fn max_extent(&self, radius: f64) -> f64 {
        match *self {
            Projection4::Orthographic => radius,
            Projection4::Perspective { eye_w } if eye_w > radius => {
                radius * eye_w / (eye_w * eye_w - radius * radius).sqrt()
            }
            Projection4::Perspective { .. } => f64::INFINITY,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perspective_formula() {
        let proj = Projection4::Perspective { eye_w: 4.0 };
        let p = proj.project(Vec4::new(1.0, -2.0, 3.0, 2.0)).unwrap();
        assert_eq!(p, Vec3::new(2.0, -4.0, 6.0)); // scale 4 / (4 - 2) = 2
        let q = proj.project(Vec4::new(1.0, 1.0, 1.0, 0.0)).unwrap();
        assert_eq!(q, Vec3::new(1.0, 1.0, 1.0)); // w = 0 is unchanged
        let r = proj.project(Vec4::new(1.0, 1.0, 1.0, -4.0)).unwrap();
        assert_eq!(r, Vec3::new(0.5, 0.5, 0.5)); // further away is smaller
    }

    #[test]
    fn at_or_behind_the_eye_is_none() {
        let proj = Projection4::Perspective { eye_w: 3.0 };
        assert_eq!(proj.project(Vec4::new(1.0, 0.0, 0.0, 3.0)), None);
        assert_eq!(proj.project(Vec4::new(1.0, 0.0, 0.0, 3.5)), None);
        assert!(proj.project(Vec4::new(1.0, 0.0, 0.0, 2.99)).is_some());
    }

    #[test]
    fn max_extent_is_attained_and_never_exceeded() {
        let (r, e) = (2.0, 2.2);
        let proj = Projection4::Perspective { eye_w: e };
        let bound = proj.max_extent(r);
        // Attained at w = r²/e with the rest of the norm in x.
        let w = r * r / e;
        let at = proj
            .project(Vec4::new((r * r - w * w).sqrt(), 0.0, 0.0, w))
            .unwrap();
        assert!((at.norm() - bound).abs() < 1e-9);
        // Never exceeded on a sweep of the sphere's (|xyz|, w) profile.
        for k in 0..=2000 {
            let w = -r + 2.0 * r * f64::from(k) / 2000.0;
            let p = Vec4::new((r * r - w * w).max(0.0).sqrt(), 0.0, 0.0, w);
            assert!(proj.project(p).unwrap().norm() <= bound + 1e-9);
        }
        assert_eq!(Projection4::Orthographic.max_extent(r), r);
        assert!(Projection4::Perspective { eye_w: r }
            .max_extent(r)
            .is_infinite());
    }

    #[test]
    fn orthographic_drops_w() {
        let p = Projection4::Orthographic
            .project(Vec4::new(1.0, 2.0, 3.0, 100.0))
            .unwrap();
        assert_eq!(p, Vec3::new(1.0, 2.0, 3.0));
    }
}
