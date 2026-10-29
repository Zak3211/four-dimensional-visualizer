//! 4D geometry: vectors, rotations, projections, meshes and the projection pipeline.
//!
//! Pure f64 maths with no dependencies. The pipeline is
//! 4D vertex → [`Mat4`] rotation → `Projection4` to 3D → `OrbitCamera` to 2D.

pub mod rotation;
pub mod vec;

pub use rotation::{Mat4, Plane};
pub use vec::{Vec3, Vec4};
