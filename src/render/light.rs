use std::f32::consts::TAU;

use crate::color::Color;
use crate::math::{
    Real,
    onb::Onb,
    sampling::{uniform_cone, uniform_sphere},
    vec3::Vec3,
};

pub struct LightSample {
    pub direction: Vec3,
    pub distance: Real,
    pub irradiance: Color,
}

#[derive(Debug, Clone, Copy)]
pub enum Light {
    Sun(Sun),
    Point(PointLight),
}

impl Light {
    pub fn sample(&self, point: Vec3, u: Real, v: Real) -> Option<LightSample> {
        match self {
            Light::Sun(sun) => Some(LightSample {
                direction: sun.sample_direction(u, v),
                distance: Real::INFINITY,
                irradiance: sun.irradiance(),
            }),
            Light::Point(light) => light.sample(point, u, v),
        }
    }

    pub fn radiance(&self, dir: Vec3) -> Color {
        match self {
            Light::Sun(sun) => sun.radiance(dir),
            Light::Point(_) => Color::BLACK,
        }
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    position: Vec3,
    intensity: Color,
    radius: Real,
}

impl PointLight {
    pub fn new(position: Vec3, color: Color, strength: Real, radius: Real) -> Self {
        Self {
            position,
            intensity: color * strength,
            radius,
        }
    }

    fn sample(&self, point: Vec3, u: Real, v: Real) -> Option<LightSample> {
        let target = self.position + self.radius * uniform_sphere(u, v);
        let to_light = target - point;
        let distance_squared = to_light.length_squared();
        if distance_squared < 1e-8 {
            return None;
        }
        let distance = distance_squared.sqrt();
        Some(LightSample {
            direction: to_light / distance,
            distance,
            irradiance: self.intensity / distance_squared,
        })
    }
}
