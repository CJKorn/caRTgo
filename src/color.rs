use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::math::{Real, vec3::Vec3};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Color(Vec3);

impl Color {
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0);

    pub const fn new(r: Real, g: Real, b: Real) -> Self {
        Self(Vec3::new(r, g, b))
    }

    pub fn r(self) -> Real {
        self.0.x()
    }

    pub fn g(self) -> Real {
        self.0.y()
    }

    pub fn b(self) -> Real {
        self.0.z()
    }

    pub fn to_vec3(self) -> Vec3 {
        self.0
    }

    pub fn from_vec3(v: Vec3) -> Self {
        Self(v)
    }

    pub fn lerp(self, other: Self, t: Real) -> Self {
        self * (1.0 - t) + other * t
    }

    pub fn to_rgb8(self) -> [u8; 3] {
        fn channel(x: Real) -> u8 {
            let gamma = if x > 0.0 { x.sqrt() } else { 0.0 };
            (256.0 * gamma.clamp(0.0, 0.999)) as u8
        }
        [channel(self.r()), channel(self.g()), channel(self.b())]
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self::new(self.r() * rhs.r(), self.g() * rhs.g(), self.b() * rhs.b())
    }
}

impl MulAssign for Color {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Mul<Real> for Color {
    type Output = Self;

    fn mul(self, rhs: Real) -> Self {
        Self(self.0 * rhs)
    }
}

impl Mul<Color> for Real {
    type Output = Color;

    fn mul(self, rhs: Color) -> Color {
        rhs * self
    }
}

impl MulAssign<Real> for Color {
    fn mul_assign(&mut self, rhs: Real) {
        *self = *self * rhs;
    }
}

impl Div<Real> for Color {
    type Output = Self;

    fn div(self, rhs: Real) -> Self {
        Self(self.0 / rhs)
    }
}

impl DivAssign<Real> for Color {
    fn div_assign(&mut self, rhs: Real) {
        *self = *self / rhs;
    }
}

impl Sum for Color {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |acc, c| acc + c)
    }
}
