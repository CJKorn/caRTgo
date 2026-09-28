use std::f32::consts::TAU;

use crate::color::Color;
use crate::math::{Real, onb::Onb, sampling::uniform_cone, vec3::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Sky {
    pub horizon: Color,
    pub zenith: Color,
}

impl Sky {
    pub fn color(&self, dir: Vec3) -> Color {
        let up = dir.normalize().z().max(0.0);
        self.horizon.lerp(self.zenith, up)
    }
}

impl Default for Sky {
    fn default() -> Self {
        Self {
            horizon: Color::WHITE,
            zenith: Color::new(0.5, 0.7, 1.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sun {
    direction: Vec3,
    irradiance: Color,
    cos_half_angle: Real,
    basis: Onb,
}

impl Sun {
    pub fn new(direction: Vec3, color: Color, strength: Real, angle: Real) -> Self {
        let direction = direction.normalize();
        Self {
            direction,
            irradiance: color * strength,
            cos_half_angle: (angle.to_radians() / 2.0).cos(),
            basis: Onb::from_w(direction),
        }
    }

    pub fn irradiance(&self) -> Color {
        self.irradiance
    }

    pub fn sample_direction(&self, u: Real, v: Real) -> Vec3 {
        self.basis.to_world(uniform_cone(u, v, self.cos_half_angle))
    }

    pub fn radiance(&self, dir: Vec3) -> Color {
        if self.cos_half_angle >= 1.0 || dir.normalize().dot(self.direction) < self.cos_half_angle {
            return Color::BLACK;
        }
        let solid_angle = TAU * (1.0 - self.cos_half_angle);
        self.irradiance / solid_angle
    }
}
