//! Triangle meshes living in 4D.

use crate::vec::Vec4;
use std::collections::BTreeMap;

/// A triangulated 2D surface in 4D, e.g. the 2-faces of a 4-polytope.
///
/// Each polygon face is split into triangles; `face_of_triangle[t]` records
/// which polygon triangle `t` came from, so the triangulation's internal
/// diagonals can be told apart from real edges.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh4 {
    pub vertices: Vec<Vec4>,
    pub triangles: Vec<[usize; 3]>,
    pub face_of_triangle: Vec<usize>,
}

impl Mesh4 {
    /// Number of polygon faces (`max(face_of_triangle) + 1`).
    pub fn face_count(&self) -> usize {
        self.face_of_triangle.iter().max().map_or(0, |&f| f + 1)
    }

    /// The real polygon edges, as sorted `(a, b)` pairs with `a < b`.
    ///
    /// A triangulation diagonal lies inside one face, so that face uses it in
    /// two triangles. A real edge lies on a face's boundary, so some face uses
    /// it in exactly one triangle. Edges are kept by that test, which needs no
    /// assumption about how many faces meet at an edge.
    pub fn feature_edges(&self) -> Vec<(usize, usize)> {
        // edge -> (face -> number of that face's triangles using the edge)
        let mut uses: BTreeMap<(usize, usize), BTreeMap<usize, u32>> = BTreeMap::new();
        for (tri, &face) in self.triangles.iter().zip(&self.face_of_triangle) {
            for k in 0..3 {
                let (a, b) = (tri[k], tri[(k + 1) % 3]);
                let edge = (a.min(b), a.max(b));
                *uses.entry(edge).or_default().entry(face).or_default() += 1;
            }
        }
        uses.into_iter()
            .filter(|(_, faces)| faces.values().any(|&n| n == 1))
            .map(|(edge, _)| edge)
            .collect()
    }

    /// Largest vertex distance from the origin. Rotations preserve norms, so
    /// no rotated vertex can have `|w| > radius()`.
    pub fn radius(&self) -> f64 {
        self.vertices.iter().map(|v| v.norm()).fold(0.0, f64::max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Mesh4 {
        Mesh4 {
            vertices: vec![
                Vec4::new(0.0, 0.0, 0.0, 0.0),
                Vec4::new(1.0, 0.0, 0.0, 0.0),
                Vec4::new(1.0, 1.0, 0.0, 0.0),
                Vec4::new(0.0, 1.0, 0.0, 0.0),
            ],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            face_of_triangle: vec![0, 0],
        }
    }

    #[test]
    fn open_square_keeps_its_boundary_and_drops_the_diagonal() {
        let m = square();
        assert_eq!(m.face_count(), 1);
        assert_eq!(m.feature_edges(), vec![(0, 1), (0, 3), (1, 2), (2, 3)]);
    }

    #[test]
    fn edge_shared_by_two_triangular_faces_is_kept() {
        let mut m = square();
        m.face_of_triangle = vec![0, 1]; // two triangles, not one square
        assert_eq!(m.feature_edges().len(), 5);
    }

    #[test]
    fn radius_and_empty_mesh() {
        assert!((square().radius() - 2f64.sqrt()).abs() < 1e-12);
        let empty = Mesh4::default();
        assert_eq!((empty.face_count(), empty.radius()), (0, 0.0));
        assert!(empty.feature_edges().is_empty());
    }
}
