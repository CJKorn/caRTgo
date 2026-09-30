use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Sub, SubAssign};

use super::Real;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    e: [Real; 3],
}

impl Vec3 {
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    pub const fn new(x: Real, y: Real, z: Real) -> Self {
        Self { e: [x, y, z] }
    }

    pub fn x(self) -> Real {
        self.e[0]
    }

    pub fn y(self) -> Real {
        self.e[1]
    }

    pub fn z(self) -> Real {
        self.e[2]
    }

    pub fn length(self) -> Real {
        self.length_squared().sqrt()
    }

    pub fn length_squared(self) -> Real {
        self.e[0] * self.e[0] + self.e[1] * self.e[1] + self.e[2] * self.e[2]
    }

    pub fn dot(self, other: Self) -> Real {
        self.e[0] * other.e[0] + self.e[1] * other.e[1] + self.e[2] * other.e[2]
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            e: [
                self.e[1] * other.e[2] - self.e[2] * other.e[1],
                self.e[2] * other.e[0] - self.e[0] * other.e[2],
                self.e[0] * other.e[1] - self.e[1] * other.e[0],
            ],
        }
    }

    pub fn mul_elem(self, other: Self) -> Self {
        Self::new(self.e[0] * other.e[0], self.e[1] * other.e[1], self.e[2] * other.e[2])
    }

    pub fn recip(self) -> Self {
        Self::new(1.0 / self.e[0], 1.0 / self.e[1], 1.0 / self.e[2])
    }

    pub fn normalize(self) -> Self {
        let len = self.length();
        if len > 0.0 {
            self / len
        }
        else {
            self
        }
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("(")?;
        fmt::Display::fmt(&self.e[0], f)?;
        f.write_str(", ")?;
        fmt::Display::fmt(&self.e[1], f)?;
        f.write_str(", ")?;
        fmt::Display::fmt(&self.e[2], f)?;
        f.write_str(")")
    }
}

impl Index<usize> for Vec3 {
    type Output = Real;

    fn index(&self, axis: usize) -> &Real {
        &self.e[axis]
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self::new(
            self.e[0] + rhs.e[0],
            self.e[1] + rhs.e[1],
            self.e[2] + rhs.e[2],
        )
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self::new(
            self.e[0] - rhs.e[0],
            self.e[1] - rhs.e[1],
            self.e[2] - rhs.e[2],
        )
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self {
        Self::new(-self.e[0], -self.e[1], -self.e[2])
    }
}

impl Mul<Real> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Real) -> Self {
        Self::new(self.e[0] * rhs, self.e[1] * rhs, self.e[2] * rhs)
    }
}

impl Mul<Vec3> for Real {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        rhs * self
    }
}

impl MulAssign<Real> for Vec3 {
    fn mul_assign(&mut self, rhs: Real) {
        *self = *self * rhs;
    }
}

impl Div<Real> for Vec3 {
    type Output = Self;

    fn div(self, rhs: Real) -> Self {
        self * (1.0 / rhs)
    }
}

impl DivAssign<Real> for Vec3 {
    fn div_assign(&mut self, rhs: Real) {
        *self = *self / rhs;
    }
}

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |acc, v| acc + v)
    }
}
