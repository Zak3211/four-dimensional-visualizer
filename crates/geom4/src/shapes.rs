//! Concrete 4D shapes.

use crate::mesh::{Mesh4, VertexId};
use crate::vec::Vec4;

/// The tesseract `[-1, 1]^4`: 16 vertices, 24 square faces (48 triangles).
///
/// Vertex `i` has coordinate `k` equal to `+1` when bit `k` of `i` is set,
/// else `-1` (x = bit 0 … w = bit 3).
pub fn tesseract() -> Mesh4 {
    let vertices = (0..16usize)
        .map(|i| {
            let c = |k: usize| if i >> k & 1 == 1 { 1.0 } else { -1.0 };
            Vec4::new(c(0), c(1), c(2), c(3))
        })
        .collect();

    let mut faces = Vec::with_capacity(24);
    // A square face frees two axes (a, b) and fixes the other two.
    for a in 0..4 {
        for b in a + 1..4 {
            let fixed: Vec<usize> = (0..4).filter(|&k| k != a && k != b).collect();
            for bits in 0..4usize {
                let base = (bits & 1) << fixed[0] | (bits >> 1 & 1) << fixed[1];
                // Corners in cyclic order around the square.
                let corners = [base, base | 1 << a, base | 1 << a | 1 << b, base | 1 << b];
                faces.push(corners.map(VertexId).to_vec());
            }
        }
    }

    Mesh4::from_polygons(vertices, faces).expect("tesseract faces are valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Edge;
    use std::collections::{BTreeMap, BTreeSet};

    const EPS: f64 = 1e-12;

    fn face_vertices(m: &Mesh4) -> Vec<BTreeSet<VertexId>> {
        m.faces()
            .iter()
            .map(|p| p.corners().iter().copied().collect())
            .collect()
    }

    #[test]
    fn counts() {
        let m = tesseract();
        assert_eq!(m.vertices().len(), 16);
        assert_eq!(m.triangles().count(), 48);
        assert_eq!(m.faces().len(), 24);
        assert!((m.radius() - 2.0).abs() < EPS);
        let distinct: BTreeSet<_> = m.vertices().iter().map(|v| format!("{v:?}")).collect();
        assert_eq!(distinct.len(), 16);
    }

    #[test]
    fn every_face_is_a_flat_square_of_side_two() {
        let m = tesseract();
        for verts in face_vertices(&m) {
            let pts: Vec<Vec4> = verts.iter().map(|&i| m.vertex(i)).collect();
            assert_eq!(pts.len(), 4);
            // Flat: exactly two coordinates are constant across the face.
            let constant = (0..4)
                .filter(|&k| pts.iter().all(|p| p.to_array()[k] == pts[0].to_array()[k]))
                .count();
            assert_eq!(constant, 2);
            // Square of side 2: four sides of length 2, two diagonals of 2√2.
            let mut d: Vec<f64> = Vec::new();
            for i in 0..4 {
                for j in i + 1..4 {
                    d.push((pts[i] - pts[j]).norm());
                }
            }
            d.sort_by(f64::total_cmp);
            for s in &d[..4] {
                assert!((s - 2.0).abs() < EPS);
            }
            for s in &d[4..] {
                assert!((s - 8f64.sqrt()).abs() < EPS);
            }
        }
    }

    #[test]
    fn corners_are_in_cyclic_order() {
        // Adjacent corners (including last to first) are joined by a side of length 2.
        let m = tesseract();
        for p in m.faces() {
            let c = p.corners();
            for i in 0..c.len() {
                let side = m.vertex(c[i]) - m.vertex(c[(i + 1) % c.len()]);
                assert!((side.norm() - 2.0).abs() < EPS);
            }
        }
    }

    #[test]
    fn edges_are_the_32_cube_edges() {
        let m = tesseract();
        let edges = m.edges();
        assert_eq!(edges.len(), 32);

        let mut degree = [0; 16];
        for e in &edges {
            let (a, b) = e.ends();
            assert!(((m.vertex(a) - m.vertex(b)).norm() - 2.0).abs() < EPS);
            degree[a.0] += 1;
            degree[b.0] += 1;
        }
        assert!(degree.iter().all(|&d| d == 4));

        // Every edge lies in exactly three square faces.
        let faces = face_vertices(&m);
        let mut faces_per_edge: BTreeMap<Edge, usize> = BTreeMap::new();
        for &e in &edges {
            let (a, b) = e.ends();
            let n = faces
                .iter()
                .filter(|f| f.contains(&a) && f.contains(&b))
                .count();
            faces_per_edge.insert(e, n);
        }
        assert!(faces_per_edge.values().all(|&n| n == 3));
    }
}
