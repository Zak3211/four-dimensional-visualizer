//! Small f64 vectors in 3 and 4 dimensions.

use std::ops::{Add, Mul, Neg, Sub};

/// A point or direction in 3D.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A point or direction in 4D.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }
}

impl Vec4 {
    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }

    pub fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// Components as an array `[x, y, z, w]`.
    pub const fn to_array(self) -> [f64; 4] {
        [self.x, self.y, self.z, self.w]
    }

    pub const fn from_array(a: [f64; 4]) -> Self {
        Self::new(a[0], a[1], a[2], a[3])
    }
}

macro_rules! impl_ops {
    ($t:ident { $($f:ident),+ }) => {
        impl Add for $t {
            type Output = $t;
            fn add(self, rhs: $t) -> $t {
                $t { $($f: self.$f + rhs.$f),+ }
            }
        }
        impl Sub for $t {
            type Output = $t;
            fn sub(self, rhs: $t) -> $t {
                $t { $($f: self.$f - rhs.$f),+ }
            }
        }
        impl Mul<f64> for $t {
            type Output = $t;
            fn mul(self, s: f64) -> $t {
                $t { $($f: self.$f * s),+ }
            }
        }
        impl Neg for $t {
            type Output = $t;
            fn neg(self) -> $t {
                $t { $($f: -self.$f),+ }
            }
        }
    };
}

impl_ops!(Vec3 { x, y, z });
impl_ops!(Vec4 { x, y, z, w });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
        let b = Vec4::new(0.5, -1.0, 2.0, 0.0);
        assert_eq!(a + b, Vec4::new(1.5, 1.0, 5.0, 4.0));
        assert_eq!(a - b, Vec4::new(0.5, 3.0, 1.0, 4.0));
        assert_eq!(a * 2.0, Vec4::new(2.0, 4.0, 6.0, 8.0));
        assert_eq!(-b, Vec4::new(-0.5, 1.0, -2.0, -0.0));
        assert_eq!(a.dot(b), 0.5 - 2.0 + 6.0);
        assert_eq!(Vec3::new(3.0, 4.0, 0.0).norm(), 5.0);
        assert_eq!(Vec4::new(1.0, 1.0, 1.0, 1.0).norm(), 2.0);
    }
}
