use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, TAU};

use crate::math::{Real, vec3::Vec3};

pub fn uniform_sphere(u: Real, v: Real) -> Vec3 {
    let z = 1.0 - 2.0 * u;
    let r = (1.0 - z * z).max(0.0).sqrt();
    let (sin, cos) = (TAU * v).sin_cos();
    Vec3::new(r * cos, r * sin, z)
}

pub fn uniform_cone(u: Real, v: Real, cos_max: Real) -> Vec3 {
    let z = 1.0 - u * (1.0 - cos_max);
    let r = (1.0 - z * z).max(0.0).sqrt();
    let (sin, cos) = (TAU * v).sin_cos();
    Vec3::new(r * cos, r * sin, z)
}

// Shirley-Chiu concentric mapping
// Needed for blue noise in the future
pub fn concentric_disk(u: Real, v: Real) -> (Real, Real) {
    let ox = 2.0 * u - 1.0;
    let oy = 2.0 * v - 1.0;
    if ox == 0.0 && oy == 0.0 {
        return (0.0, 0.0);
    }

    let (r, theta) = if ox.abs() > oy.abs() {
        (ox, FRAC_PI_4 * (oy / ox))
    }
    else {
        (oy, FRAC_PI_2 - FRAC_PI_4 * (ox / oy))
    };
    (r * theta.cos(), r * theta.sin())
}
