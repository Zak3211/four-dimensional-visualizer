//! The full pipeline: rotate in 4D, project to 3D, project to the screen.

use crate::camera::{OrbitCamera, ScreenPoint};
use crate::mesh::{FaceId, Mesh4, VertexId};
use crate::projection::Projection4;
use crate::rotation::Mat4;

/// A mesh's vertices after the whole pipeline, indexed by [`VertexId`].
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedMesh {
    points: Vec<Option<ScreenPoint>>,
    vertex_w: Vec<f64>,
}

impl ProjectedMesh {
    /// The vertex on screen, or `None` when it is behind the 4D eye or the camera.
    pub fn point(&self, id: VertexId) -> Option<ScreenPoint> {
        self.points[id.0]
    }

    /// The vertex's w after rotation, before projection (used for colour).
    pub fn w(&self, id: VertexId) -> f64 {
        self.vertex_w[id.0]
    }
}

/// Runs every vertex of `mesh` through rotation, 4D→3D and 3D→2D projection.
pub fn project_mesh(
    mesh: &Mesh4,
    rot: &Mat4,
    proj: &Projection4,
    cam: &OrbitCamera,
    aspect: f64,
) -> ProjectedMesh {
    let (points, vertex_w) = mesh
        .vertices()
        .iter()
        .map(|&v| {
            let r = *rot * v;
            let p = proj.project(r).and_then(|p3| cam.project(p3, aspect));
            (p, r.w)
        })
        .unzip();
    ProjectedMesh { points, vertex_w }
}

/// Faces for the painter's algorithm, furthest first by the mean depth of
/// their corners. Faces with any corner off screen (`None`) are left out.
pub fn back_to_front(mesh: &Mesh4, pm: &ProjectedMesh) -> Vec<FaceId> {
    let mut keyed: Vec<(f64, FaceId)> = mesh
        .face_ids()
        .filter_map(|f| {
            let corners = mesh.face(f).corners();
            let sum = corners
                .iter()
                .map(|&v| pm.point(v).map(|p| p.depth))
                .sum::<Option<f64>>()?;
            Some((sum / corners.len() as f64, f))
        })
        .collect();
    // Furthest first; ties broken by face id so the order is deterministic.
    keyed.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    keyed.into_iter().map(|(_, f)| f).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::tesseract;

    fn rotations() -> Vec<Mat4> {
        vec![
            Mat4::identity(),
            Mat4::from_angles(&[0.4, 1.1, -0.7, 2.0, 0.3, -1.5]),
            Mat4::from_angles(&[0.0, 0.0, std::f64::consts::FRAC_PI_4, 0.0, 0.0, 0.0]),
        ]
    }

    #[test]
    fn eye_beyond_radius_projects_every_vertex() {
        let m = tesseract();
        let proj = Projection4::Perspective {
            eye_w: 1.1 * m.radius(),
        };
        let cam = OrbitCamera::default();
        // Generic rotations; the exact w = radius case has its own test.
        let mut rots = rotations();
        for a in [0.5, 1.0, 1.5] {
            rots.push(Mat4::from_angles(&[0.0, 0.0, a, 0.0, a, a]));
        }
        for rot in &rots {
            let pm = project_mesh(&m, rot, &proj, &cam, 1.5);
            assert!(pm.points.iter().all(Option::is_some));
            assert_eq!(back_to_front(&m, &pm).len(), m.faces().len());
        }
    }

    #[test]
    fn back_to_front_orders_faces_furthest_first() {
        let m = tesseract();
        for rot in rotations() {
            let pm = project_mesh(
                &m,
                &rot,
                &Projection4::default(),
                &OrbitCamera::default(),
                1.0,
            );
            let face_depth = |f: FaceId| {
                let c = m.face(f).corners();
                c.iter().map(|&v| pm.point(v).unwrap().depth).sum::<f64>() / c.len() as f64
            };
            let order = back_to_front(&m, &pm);
            assert_eq!(order.len(), 24);
            assert!(order
                .windows(2)
                .all(|w| face_depth(w[0]) >= face_depth(w[1])));
        }
    }

    #[test]
    fn vertex_exactly_at_w_equal_radius_still_projects() {
        let m = tesseract();
        // Orthonormal rows; the last sends (1, 1, 1, 1) to w = 2 = radius.
        let rot = Mat4([
            [0.5, -0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5, 0.5],
        ]);
        let proj = Projection4::Perspective {
            eye_w: 1.1 * m.radius(),
        };
        let pm = project_mesh(&m, &rot, &proj, &OrbitCamera::default(), 1.0);
        assert!(pm.vertex_w.iter().any(|&w| (w - m.radius()).abs() < 1e-12));
        assert!(pm.points.iter().all(Option::is_some));
    }

    #[test]
    fn faces_with_a_hidden_vertex_are_skipped() {
        let m = tesseract();
        // Eye at w = 0.5: the eight vertices with w = +1 are behind it.
        let proj = Projection4::Perspective { eye_w: 0.5 };
        let pm = project_mesh(&m, &Mat4::identity(), &proj, &OrbitCamera::default(), 1.0);
        assert_eq!(pm.points.iter().filter(|p| p.is_none()).count(), 8);
        let order = back_to_front(&m, &pm);
        // Only the 6 squares of the w = -1 cube survive.
        assert_eq!(order.len(), 6);
        for f in order {
            assert!(m.face(f).corners().iter().all(|&v| pm.point(v).is_some()));
        }
    }

    #[test]
    fn vertex_w_is_after_rotation() {
        let m = tesseract();
        let rot = Mat4::from_angles(&[0.0, 0.0, 0.9, 0.0, 0.0, 0.0]);
        let pm = project_mesh(
            &m,
            &rot,
            &Projection4::Orthographic,
            &OrbitCamera::default(),
            1.0,
        );
        for (v, w) in m.vertices().iter().zip(&pm.vertex_w) {
            assert_eq!(*w, (rot * *v).w);
        }
    }
}
