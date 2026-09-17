use crate::math::{Real, vec3::Vec3};

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
    inverse_direction: Vec3,
    time: Real,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3, time: Real) -> Self {
        Self {
            origin,
            direction,
            inverse_direction: direction.recip(),
            time,
        }
    }

    pub fn origin(self) -> Vec3 {
        self.origin
    }

    pub fn direction(self) -> Vec3 {
        self.direction
    }

    pub fn inverse_direction(self) -> Vec3 {
        self.inverse_direction
    }

    pub fn time(self) -> Real {
        self.time
    }

    pub fn at(self, t: Real) -> Vec3 {
        self.origin + self.direction * t
    }
}