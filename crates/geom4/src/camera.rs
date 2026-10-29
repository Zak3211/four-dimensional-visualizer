//! An ordinary 3D orbit camera that projects the 3D "screen" onto 2D.

use crate::vec::Vec3;

/// Points closer than this to the camera plane are treated as behind it.
const NEAR: f64 = 1e-3;

/// A camera orbiting the origin and always looking at it.
///
/// `yaw` turns around the vertical (y) axis, `pitch` tilts up from the
/// horizontal plane, both in radians. `distance` is from the origin, and
/// `fov_y` is the full vertical field of view in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitCamera {
    pub yaw: f64,
    pub pitch: f64,
    pub distance: f64,
    pub fov_y: f64,
}

/// A projected point. `x` and `y` are in `[-1, 1]` when on screen (y up);
/// `depth` is the distance along the view direction (larger is further).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
    pub depth: f64,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.6,
            pitch: 0.35,
            distance: 8.0,
            fov_y: 45f64.to_radians(),
        }
    }
}

impl OrbitCamera {
    /// Camera position in world space.
    pub fn position(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(cp * sy, sp, cp * cy) * self.distance
    }

    /// Projects `p` for a viewport of width/height ratio `aspect`.
    /// `None` when `p` is at or behind the camera.
    pub fn project(&self, p: Vec3, aspect: f64) -> Option<ScreenPoint> {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        // Orthonormal camera basis, written out from yaw and pitch so it never
        // degenerates (no cross product with a world "up").
        let forward = Vec3::new(-cp * sy, -sp, -cp * cy);
        let right = Vec3::new(cy, 0.0, -sy);
        let up = Vec3::new(-sy * sp, cp, -cy * sp);

        let rel = p - self.position();
        let depth = rel.dot(forward);
        if depth <= NEAR {
            return None;
        }
        let f = 1.0 / (self.fov_y / 2.0).tan();
        Some(ScreenPoint {
            x: rel.dot(right) * f / (aspect * depth),
            y: rel.dot(up) * f / depth,
            depth,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-12;

    fn cams() -> Vec<OrbitCamera> {
        vec![
            OrbitCamera::default(),
            OrbitCamera {
                yaw: -2.0,
                pitch: -1.2,
                distance: 3.0,
                fov_y: 1.0,
            },
            OrbitCamera {
                yaw: 0.0,
                pitch: std::f64::consts::FRAC_PI_2, // looking straight down
                distance: 5.0,
                fov_y: 0.8,
            },
        ]
    }

    #[test]
    fn origin_lands_at_screen_centre() {
        for cam in cams() {
            let s = cam.project(Vec3::default(), 1.6).unwrap();
            assert!(s.x.abs() < EPS && s.y.abs() < EPS, "{s:?}");
            assert!((s.depth - cam.distance).abs() < EPS);
        }
    }

    #[test]
    fn points_behind_the_camera_are_none() {
        for cam in cams() {
            let pos = cam.position();
            assert_eq!(cam.project(pos * 1.5, 1.0), None); // behind
            assert_eq!(cam.project(pos, 1.0), None); // at the camera
        }
    }

    #[test]
    fn screen_axes_point_right_and_up() {
        let cam = OrbitCamera {
            yaw: 0.0,
            pitch: 0.0,
            distance: 5.0,
            fov_y: std::f64::consts::FRAC_PI_2, // f = 1
        };
        // Camera at +z looking at the origin: +x is right, +y is up.
        let r = cam.project(Vec3::new(1.0, 0.0, 0.0), 2.0).unwrap();
        assert!((r.x - 1.0 / (2.0 * 5.0)).abs() < EPS && r.y.abs() < EPS);
        let u = cam.project(Vec3::new(0.0, 5.0, 0.0), 1.0).unwrap();
        assert!(u.x.abs() < EPS && (u.y - 1.0).abs() < EPS); // top edge of the fov
    }

    #[test]
    fn further_points_have_larger_depth() {
        let cam = OrbitCamera::default();
        let near = cam.project(cam.position() * 0.5, 1.0).unwrap();
        let far = cam.project(cam.position() * -0.5, 1.0).unwrap();
        assert!(far.depth > near.depth);
    }
}
