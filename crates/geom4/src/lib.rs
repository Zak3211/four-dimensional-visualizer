//! 4D geometry: vectors, rotations, projections, meshes and the projection pipeline.
//!
//! Pure f64 maths with no dependencies. The pipeline is
//! 4D vertex → [`Mat4`] rotation → [`Projection4`] to 3D → [`OrbitCamera`] to 2D,
//! run over a whole [`Mesh4`] by [`project_mesh`] and ordered by [`back_to_front`].

pub mod camera;
pub mod mesh;
pub mod pipeline;
pub mod projection;
pub mod rotation;
pub mod shapes;
pub mod vec;

pub use camera::{OrbitCamera, ScreenPoint};
pub use mesh::Mesh4;
pub use pipeline::{back_to_front, project_mesh, ProjectedMesh};
pub use projection::Projection4;
pub use rotation::{Mat4, Plane};
pub use shapes::tesseract;
pub use vec::{Vec3, Vec4};
