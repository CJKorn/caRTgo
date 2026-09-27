use std::ops::Mul;

use crate::math::{Real, vec3::Vec3};

// w + xi + yj + zk
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    w: Real,
    x: Real,
    y: Real,
    z: Real,
}

impl Quat {
    pub const IDENTITY: Self = Self::new(1.0, 0.0, 0.0, 0.0);

    pub const fn new(w: Real, x: Real, y: Real, z: Real) -> Self {
        Self { w, x, y, z }
    }

    pub fn from_axis_angle(axis: Vec3, angle: Real) -> Self {
        let axis = axis.normalize();
        let (sin, cos) = (angle / 2.0).sin_cos();
        Self::new(cos, axis.x() * sin, axis.y() * sin, axis.z() * sin)
    }

    pub fn from_euler_xyz(angles: Vec3) -> Self {
        Self::from_axis_angle(Vec3::Z, angles.z())
            * Self::from_axis_angle(Vec3::Y, angles.y())
            * Self::from_axis_angle(Vec3::X, angles.x())
    }

    pub fn w(self) -> Real {
        self.w
    }

    pub fn x(self) -> Real {
        self.x
    }

    pub fn y(self) -> Real {
        self.y
    }

    pub fn z(self) -> Real {
        self.z
    }

    pub fn conjugate(self) -> Self {
        Self::new(self.w, -self.x, -self.y, -self.z)
    }

    pub fn normalize(self) -> Self {
        let len = (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Self::new(self.w / len, self.x / len, self.y / len, self.z / len)
    }

    pub fn rotate(self, v: Vec3) -> Vec3 {
        let q = Vec3::new(self.x, self.y, self.z);
        let t = 2.0 * q.cross(v);
        v + self.w * t + q.cross(t)
    }
}

impl Default for Quat {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Quat {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        )
    }
}
