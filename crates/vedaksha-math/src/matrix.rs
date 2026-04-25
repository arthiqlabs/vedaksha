// Copyright © 2026 ArthIQ Labs LLC. All rights reserved.
// Vedākṣha — Vision from Vedas
// Licensed under BSL 1.1. See LICENSE file.
// Contact: info@arthiq.net | https://vedaksha.net

//! 3×3 rotation matrices for coordinate frame transformations.
//!
//! Provides matrix multiplication, transposition, and rotation about
//! the X, Y, and Z axes for astronomical coordinate conversions.
//!
//! Source: Green, "Spherical Astronomy", Ch. 2.

use libm::{cos, sin, sqrt};

/// A 3×3 matrix stored in row-major order.
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Matrix3 {
    pub data: [[f64; 3]; 3],
}

/// A 3-component vector.
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Matrix3 {
    /// The 3×3 identity matrix.
    pub const IDENTITY: Self = Self {
        data: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    /// Rotation matrix about the X axis by `theta` radians.
    ///
    /// ```text
    /// Rx = | 1    0     0  |
    ///      | 0   cos   sin |
    ///      | 0  -sin   cos |
    /// ```
    pub fn rotation_x(theta: f64) -> Self {
        let (s, c) = (sin(theta), cos(theta));
        Self {
            data: [[1.0, 0.0, 0.0], [0.0, c, s], [0.0, -s, c]],
        }
    }

    /// Rotation matrix about the Y axis by `theta` radians.
    ///
    /// ```text
    /// Ry = |  cos  0  -sin |
    ///      |   0   1    0  |
    ///      |  sin  0   cos |
    /// ```
    pub fn rotation_y(theta: f64) -> Self {
        let (s, c) = (sin(theta), cos(theta));
        Self {
            data: [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]],
        }
    }

    /// Rotation matrix about the Z axis by `theta` radians.
    ///
    /// ```text
    /// Rz = |  cos  sin  0 |
    ///      | -sin  cos  0 |
    ///      |   0    0   1 |
    /// ```
    pub fn rotation_z(theta: f64) -> Self {
        let (s, c) = (sin(theta), cos(theta));
        Self {
            data: [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// Returns the transpose of this matrix.
    pub fn transpose(&self) -> Self {
        let d = &self.data;
        Self {
            data: [
                [d[0][0], d[1][0], d[2][0]],
                [d[0][1], d[1][1], d[2][1]],
                [d[0][2], d[1][2], d[2][2]],
            ],
        }
    }

    /// Returns the matrix product `self * other`.
    #[allow(clippy::needless_range_loop)]
    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = [[0.0_f64; 3]; 3];
        for r in 0..3 {
            for c in 0..3 {
                result[r][c] = self.data[r][0] * other.data[0][c]
                    + self.data[r][1] * other.data[1][c]
                    + self.data[r][2] * other.data[2][c];
            }
        }
        Self { data: result }
    }

    /// Applies this matrix to a vector, returning `self * v`.
    pub fn apply(&self, v: &Vector3) -> Vector3 {
        Vector3 {
            x: self.data[0][0] * v.x + self.data[0][1] * v.y + self.data[0][2] * v.z,
            y: self.data[1][0] * v.x + self.data[1][1] * v.y + self.data[1][2] * v.z,
            z: self.data[2][0] * v.x + self.data[2][1] * v.y + self.data[2][2] * v.z,
        }
    }
}

impl Vector3 {
    /// Creates a new `Vector3` with the given components.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns the Euclidean length (magnitude) of this vector.
    #[must_use]
    pub fn length(&self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f64::consts::{FRAC_PI_2, PI};

    const EPS: f64 = 1e-12;

    fn assert_vec_eq(a: &Vector3, b: &Vector3) {
        assert!((a.x - b.x).abs() < EPS, "x: {} != {}", a.x, b.x);
        assert!((a.y - b.y).abs() < EPS, "y: {} != {}", a.y, b.y);
        assert!((a.z - b.z).abs() < EPS, "z: {} != {}", a.z, b.z);
    }

    fn assert_mat_eq(a: &Matrix3, b: &Matrix3) {
        for r in 0..3 {
            for c in 0..3 {
                assert!(
                    (a.data[r][c] - b.data[r][c]).abs() < EPS,
                    "data[{}][{}]: {} != {}",
                    r,
                    c,
                    a.data[r][c],
                    b.data[r][c]
                );
            }
        }
    }

    #[test]
    fn identity_apply() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let result = Matrix3::IDENTITY.apply(&v);
        assert_vec_eq(&result, &v);
    }

    #[test]
    fn identity_multiply() {
        let r = Matrix3::rotation_z(0.5);
        let result = Matrix3::IDENTITY.multiply(&r);
        assert_mat_eq(&result, &r);
    }

    #[test]
    fn rotation_x_zero() {
        assert_mat_eq(&Matrix3::rotation_x(0.0), &Matrix3::IDENTITY);
    }

    #[test]
    fn rotation_x_90_rotates_y_to_neg_z() {
        let v = Vector3::new(0.0, 1.0, 0.0);
        let result = Matrix3::rotation_x(FRAC_PI_2).apply(&v);
        assert_vec_eq(&result, &Vector3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn rotation_x_preserves_length() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let result = Matrix3::rotation_x(1.23).apply(&v);
        assert!((result.length() - v.length()).abs() < EPS);
    }

    #[test]
    fn rotation_y_zero() {
        assert_mat_eq(&Matrix3::rotation_y(0.0), &Matrix3::IDENTITY);
    }

    #[test]
    fn rotation_y_90_rotates_z_to_neg_x() {
        let v = Vector3::new(0.0, 0.0, 1.0);
        let result = Matrix3::rotation_y(FRAC_PI_2).apply(&v);
        assert_vec_eq(&result, &Vector3::new(-1.0, 0.0, 0.0));
    }

    #[test]
    fn rotation_z_zero() {
        assert_mat_eq(&Matrix3::rotation_z(0.0), &Matrix3::IDENTITY);
    }

    #[test]
    fn rotation_z_90_rotates_x_to_neg_y() {
        let v = Vector3::new(1.0, 0.0, 0.0);
        let result = Matrix3::rotation_z(FRAC_PI_2).apply(&v);
        assert_vec_eq(&result, &Vector3::new(0.0, -1.0, 0.0));
    }

    #[test]
    fn transpose_identity() {
        assert_mat_eq(&Matrix3::IDENTITY.transpose(), &Matrix3::IDENTITY);
    }

    #[test]
    fn rotation_transpose_is_inverse() {
        let r = Matrix3::rotation_x(0.7);
        let result = r.multiply(&r.transpose());
        assert_mat_eq(&result, &Matrix3::IDENTITY);
    }

    #[test]
    fn double_transpose_is_original() {
        let r = Matrix3::rotation_y(1.1);
        assert_mat_eq(&r.transpose().transpose(), &r);
    }

    #[test]
    fn multiply_associative() {
        let a = Matrix3::rotation_x(0.3);
        let b = Matrix3::rotation_y(0.5);
        let c = Matrix3::rotation_z(0.7);
        let ab_c = a.multiply(&b).multiply(&c);
        let a_bc = a.multiply(&b.multiply(&c));
        assert_mat_eq(&ab_c, &a_bc);
    }

    #[test]
    fn full_rotation_is_identity() {
        assert_mat_eq(&Matrix3::rotation_z(2.0 * PI), &Matrix3::IDENTITY);
    }

    #[test]
    fn composition_equals_sum() {
        let a = 0.4_f64;
        let b = 0.6_f64;
        let composed = Matrix3::rotation_x(a).multiply(&Matrix3::rotation_x(b));
        let direct = Matrix3::rotation_x(a + b);
        assert_mat_eq(&composed, &direct);
    }

    #[test]
    fn vector_length() {
        let v = Vector3::new(3.0, 4.0, 0.0);
        assert!((v.length() - 5.0).abs() < EPS);
    }
}
