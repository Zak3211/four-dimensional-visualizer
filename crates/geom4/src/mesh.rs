//! Polygon meshes living in 4D.

use crate::vec::Vec4;
use std::collections::BTreeSet;
use std::fmt;

/// Index of a vertex in [`Mesh4::vertices`]. Not tied to a particular mesh:
/// using an id from another mesh panics or silently picks the wrong vertex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VertexId(pub usize);

/// Index of a face in [`Mesh4::faces`]. Like [`VertexId`], not tied to a mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FaceId(pub usize);

/// Three vertices of a triangle, in the winding of the polygon it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triangle(pub [VertexId; 3]);

/// An undirected edge. The ends are stored sorted, so `Edge::new(u, v) == Edge::new(v, u)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Edge {
    a: VertexId,
    b: VertexId,
}

impl Edge {
    pub fn new(u: VertexId, v: VertexId) -> Self {
        Self {
            a: u.min(v),
            b: u.max(v),
        }
    }

    /// The ends, smaller id first.
    pub fn ends(self) -> (VertexId, VertexId) {
        (self.a, self.b)
    }
}

/// A convex planar polygon face: its corners in cyclic order, and a fan
/// triangulation of them built once at construction.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    corners: Vec<VertexId>,
    triangles: Vec<Triangle>,
}

impl Polygon {
    /// Fan-triangulates from the first corner: `[c0, ci, ci+1]`.
    fn new(corners: Vec<VertexId>) -> Self {
        let triangles = (1..corners.len() - 1)
            .map(|i| Triangle([corners[0], corners[i], corners[i + 1]]))
            .collect();
        Self { corners, triangles }
    }

    /// Corners in cyclic order.
    pub fn corners(&self) -> &[VertexId] {
        &self.corners
    }

    /// `corners().len() - 2` triangles covering the polygon.
    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }

    /// The boundary edges, one per pair of cyclically adjacent corners.
    pub fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        let n = self.corners.len();
        (0..n).map(move |i| Edge::new(self.corners[i], self.corners[(i + 1) % n]))
    }
}

/// Why [`Mesh4::from_polygons`] rejected its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshError {
    IndexOutOfRange { face: FaceId, vertex: VertexId },
    TooFewCorners { face: FaceId },
    RepeatedCorner { face: FaceId, vertex: VertexId },
}

impl fmt::Display for MeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexOutOfRange { face, vertex } => {
                write!(f, "face {} uses missing vertex {}", face.0, vertex.0)
            }
            Self::TooFewCorners { face } => write!(f, "face {} has fewer than 3 corners", face.0),
            Self::RepeatedCorner { face, vertex } => {
                write!(f, "face {} repeats vertex {}", face.0, vertex.0)
            }
        }
    }
}

impl std::error::Error for MeshError {}

/// A 2D surface in 4D made of polygon faces sharing one vertex buffer,
/// e.g. the 2-faces of a 4-polytope.
///
/// Every face must be convex and planar with corners in cyclic order; this is
/// checked by each shape's tests, not here, since it needs a tolerance.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh4 {
    vertices: Vec<Vec4>,
    faces: Vec<Polygon>,
}

impl Mesh4 {
    /// Builds a mesh, triangulating each face. Checks that every face has at
    /// least 3 distinct corners, all of them existing vertices.
    pub fn from_polygons(
        vertices: Vec<Vec4>,
        faces: Vec<Vec<VertexId>>,
    ) -> Result<Self, MeshError> {
        for (f, corners) in faces.iter().enumerate() {
            let face = FaceId(f);
            if corners.len() < 3 {
                return Err(MeshError::TooFewCorners { face });
            }
            let mut seen = BTreeSet::new();
            for &vertex in corners {
                if vertex.0 >= vertices.len() {
                    return Err(MeshError::IndexOutOfRange { face, vertex });
                }
                if !seen.insert(vertex) {
                    return Err(MeshError::RepeatedCorner { face, vertex });
                }
            }
        }
        let faces = faces.into_iter().map(Polygon::new).collect();
        Ok(Self { vertices, faces })
    }

    pub fn vertices(&self) -> &[Vec4] {
        &self.vertices
    }

    pub fn faces(&self) -> &[Polygon] {
        &self.faces
    }

    /// Panics if `id` is not a vertex of this mesh.
    pub fn vertex(&self, id: VertexId) -> Vec4 {
        self.vertices[id.0]
    }

    /// Panics if `id` is not a face of this mesh.
    pub fn face(&self, id: FaceId) -> &Polygon {
        &self.faces[id.0]
    }

    /// Ids of all faces, in order.
    pub fn face_ids(&self) -> impl Iterator<Item = FaceId> {
        (0..self.faces.len()).map(FaceId)
    }

    /// Every face's boundary edges, deduplicated and sorted. Every polygon
    /// side counts, so a surface passed in already triangulated shows its
    /// diagonals as edges: pass the real polygons instead.
    pub fn edges(&self) -> Vec<Edge> {
        let set: BTreeSet<Edge> = self.faces.iter().flat_map(Polygon::edges).collect();
        set.into_iter().collect()
    }

    /// All triangles with the face each came from, faces in order.
    pub fn triangles(&self) -> impl Iterator<Item = (FaceId, Triangle)> + '_ {
        self.faces
            .iter()
            .enumerate()
            .flat_map(|(f, p)| p.triangles.iter().map(move |&t| (FaceId(f), t)))
    }

    /// Mean of the face's corners.
    pub fn face_centre(&self, id: FaceId) -> Vec4 {
        let corners = self.face(id).corners();
        let sum = corners
            .iter()
            .fold(Vec4::default(), |acc, &v| acc + self.vertex(v));
        sum * (1.0 / corners.len() as f64)
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

    fn ids(v: &[usize]) -> Vec<VertexId> {
        v.iter().copied().map(VertexId).collect()
    }

    fn square_vertices() -> Vec<Vec4> {
        vec![
            Vec4::new(0.0, 0.0, 0.0, 0.0),
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(1.0, 1.0, 0.0, 0.0),
            Vec4::new(0.0, 1.0, 0.0, 0.0),
        ]
    }

    fn square() -> Mesh4 {
        Mesh4::from_polygons(square_vertices(), vec![ids(&[0, 1, 2, 3])]).unwrap()
    }

    fn edge(a: usize, b: usize) -> Edge {
        Edge::new(VertexId(a), VertexId(b))
    }

    #[test]
    fn square_has_its_boundary_and_no_diagonal() {
        let m = square();
        assert_eq!(m.faces().len(), 1);
        assert_eq!(
            m.edges(),
            vec![edge(0, 1), edge(0, 3), edge(1, 2), edge(2, 3)]
        );
    }

    #[test]
    fn fan_triangulation_covers_the_polygon() {
        let m = square();
        let tris: Vec<_> = m.triangles().collect();
        assert_eq!(
            tris,
            vec![
                (FaceId(0), Triangle([VertexId(0), VertexId(1), VertexId(2)])),
                (FaceId(0), Triangle([VertexId(0), VertexId(2), VertexId(3)])),
            ]
        );
        assert_eq!(m.face(FaceId(0)).triangles().len(), 2);
        assert_eq!(m.face_ids().collect::<Vec<_>>(), vec![FaceId(0)]);
    }

    #[test]
    fn edge_shared_by_two_triangular_faces_is_kept_once() {
        let m = Mesh4::from_polygons(square_vertices(), vec![ids(&[0, 1, 2]), ids(&[0, 2, 3])])
            .unwrap();
        assert_eq!(m.edges().len(), 5);
        assert!(m.edges().contains(&edge(2, 0)));
    }

    #[test]
    fn edge_ends_are_sorted() {
        assert_eq!(edge(3, 1), edge(1, 3));
        assert_eq!(edge(3, 1).ends(), (VertexId(1), VertexId(3)));
    }

    #[test]
    fn bad_polygons_are_rejected() {
        let bad = |face: &[usize]| Mesh4::from_polygons(square_vertices(), vec![ids(face)]);
        assert_eq!(
            bad(&[0, 1]),
            Err(MeshError::TooFewCorners { face: FaceId(0) })
        );
        assert_eq!(
            bad(&[0, 1, 4]),
            Err(MeshError::IndexOutOfRange {
                face: FaceId(0),
                vertex: VertexId(4)
            })
        );
        assert_eq!(
            bad(&[0, 1, 2, 1]),
            Err(MeshError::RepeatedCorner {
                face: FaceId(0),
                vertex: VertexId(1)
            })
        );
    }

    #[test]
    fn face_centre_radius_and_empty_mesh() {
        let m = square();
        assert_eq!(m.face_centre(FaceId(0)), Vec4::new(0.5, 0.5, 0.0, 0.0));
        assert!((m.radius() - 2f64.sqrt()).abs() < 1e-12);
        let empty = Mesh4::default();
        assert_eq!((empty.faces().len(), empty.radius()), (0, 0.0));
        assert!(empty.edges().is_empty());
        assert_eq!(empty.triangles().count(), 0);
    }
}
