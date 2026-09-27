use crate::math::Real;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    pub min: Real,
    pub max: Real,
}

impl Interval {
    pub const EMPTY: Self = Self::new(Real::INFINITY, Real::NEG_INFINITY);
    pub const UNIVERSE: Self = Self::new(Real::NEG_INFINITY, Real::INFINITY);

    pub const fn new(min: Real, max: Real) -> Self {
        Self { min, max }
    }

    pub fn enclosing(a: Self, b: Self) -> Self {
        Self::new(a.min.min(b.min), a.max.max(b.max))
    }

    pub fn size(self) -> Real {
        self.max - self.min
    }

    pub fn contains(self, x: Real) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(self, x: Real) -> bool {
        self.min < x && x < self.max
    }

    pub fn contains_interval(self, other: Self) -> bool {
        self.min <= other.min && self.max >= other.max
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.min <= other.max && other.min <= self.max
    }

    pub fn clamp(self, x: Real) -> Real {
        if x < self.min {
            self.min
        }
        else if x > self.max {
            self.max
        }
        else {
            x
        }
    }

    pub fn expand(self, delta: Real) -> Self {
        let padding = delta / 2.0;
        Self::new(self.min - padding, self.max + padding)
    }
}
