//! The full pipeline: rotate in 4D, project to 3D, project to the screen.

use crate::camera::{OrbitCamera, ScreenPoint};
use crate::mesh::Mesh4;
use crate::projection::Projection4;
use crate::rotation::Mat4;

/// A mesh's vertices after the whole pipeline, indexed like `Mesh4::vertices`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectedMesh {
    /// `None` when the vertex is behind the 4D eye or the camera.
    pub points: Vec<Option<ScreenPoint>>,
    /// Each vertex's w after rotation, before projection (used for colour).
    pub vertex_w: Vec<f64>,
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
        .vertices
        .iter()
        .map(|&v| {
            let r = *rot * v;
            let p = proj.project(r).and_then(|p3| cam.project(p3, aspect));
            (p, r.w)
        })
        .unzip();
    ProjectedMesh { points, vertex_w }
}

/// Triangle indices ordered furthest first by mean vertex depth (painter's
/// algorithm). Triangles with any `None` vertex are left out.
pub fn back_to_front(mesh: &Mesh4, pm: &ProjectedMesh) -> Vec<usize> {
    let mut keyed: Vec<(f64, usize)> = mesh
        .triangles
        .iter()
        .enumerate()
        .filter_map(|(t, tri)| {
            let depth_sum = tri
                .iter()
                .map(|&i| pm.points[i].map(|p| p.depth))
                .sum::<Option<f64>>()?;
            Some((depth_sum / 3.0, t))
        })
        .collect();
    keyed.sort_by(|a, b| b.0.total_cmp(&a.0));
    keyed.into_iter().map(|(_, t)| t).collect()
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
        // Includes rotations that push a vertex right up to w = radius.
        let mut rots = rotations();
        for a in [0.5, 1.0, 1.5] {
            rots.push(Mat4::from_angles(&[0.0, 0.0, a, 0.0, a, a]));
        }
        for rot in &rots {
            let pm = project_mesh(&m, rot, &proj, &cam, 1.5);
            assert!(pm.points.iter().all(Option::is_some));
            assert_eq!(back_to_front(&m, &pm).len(), m.triangles.len());
        }
    }

    #[test]
    fn back_to_front_orders_furthest_first() {
        let m = tesseract();
        for rot in rotations() {
            let pm = project_mesh(
                &m,
                &rot,
                &Projection4::default(),
                &OrbitCamera::default(),
                1.0,
            );
            let mean_depth = |t: usize| {
                m.triangles[t]
                    .iter()
                    .map(|&i| pm.points[i].unwrap().depth)
                    .sum::<f64>()
                    / 3.0
            };
            let order = back_to_front(&m, &pm);
            assert_eq!(order.len(), 48);
            assert!(order
                .windows(2)
                .all(|w| mean_depth(w[0]) >= mean_depth(w[1])));
        }
    }

    #[test]
    fn triangles_with_a_hidden_vertex_are_skipped() {
        let m = tesseract();
        // Eye at w = 0.5: the eight vertices with w = +1 are behind it.
        let proj = Projection4::Perspective { eye_w: 0.5 };
        let pm = project_mesh(&m, &Mat4::identity(), &proj, &OrbitCamera::default(), 1.0);
        assert_eq!(pm.points.iter().filter(|p| p.is_none()).count(), 8);
        let order = back_to_front(&m, &pm);
        // Only the 6 squares of the w = -1 cube survive.
        assert_eq!(order.len(), 12);
        for t in order {
            assert!(m.triangles[t].iter().all(|&i| pm.points[i].is_some()));
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
        for (v, w) in m.vertices.iter().zip(&pm.vertex_w) {
            assert_eq!(*w, (rot * *v).w);
        }
    }
}
