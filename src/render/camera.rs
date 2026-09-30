use std::f32::consts::FRAC_PI_2;

use crate::math::{Real, quat::Quat, ray::Ray, sampling::concentric_disk, vec3::Vec3};
use crate::render::image_spec::ImageSpec;
use crate::rng::Pcg32;

// Z = up :)
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    position: Vec3,
    rotation: Quat,
    vfov: Real,
    focus_dist: Real,
    aperture: Real,
    aspect: Real,

    upper_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: Real,
}

impl Camera {
    pub fn new(vfov: Real, focus_dist: Real, aperture: Real, spec: &ImageSpec) -> Self {
        let mut camera = Self {
            position: Vec3::default(),
            rotation: Quat::IDENTITY,
            vfov,
            focus_dist,
            aperture,
            aspect: spec.aspect(),
            upper_left_corner: Vec3::default(),
            horizontal: Vec3::default(),
            vertical: Vec3::default(),
            u: Vec3::default(),
            v: Vec3::default(),
            lens_radius: 0.0,
        };
        camera.rebuild();
        camera
    }

    pub fn vfov(&self) -> Real {
        self.vfov
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.rebuild();
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation.normalize();
        self.rebuild();
    }

    pub fn look_at(&mut self, target: Vec3) {
        let dir = (target - self.position).normalize();
        let tilt = FRAC_PI_2 + dir.z().clamp(-1.0, 1.0).asin();
        let turn = (-dir.x()).atan2(dir.y());
        self.set_rotation(
            Quat::from_axis_angle(Vec3::Z, turn) * Quat::from_axis_angle(Vec3::X, tilt),
        );
    }

    fn rebuild(&mut self) {
        let theta = self.vfov.to_radians();
        let viewport_height = 2.0 * (theta / 2.0).tan() * self.focus_dist;
        let viewport_width = self.aspect * viewport_height;

        self.u = self.rotation.rotate(Vec3::X);
        self.v = self.rotation.rotate(Vec3::Y);
        let w = self.rotation.rotate(Vec3::Z);

        self.horizontal = viewport_width * self.u;
        self.vertical = -viewport_height * self.v;
        self.upper_left_corner = self.position - self.focus_dist * w
            - self.horizontal / 2.0
            - self.vertical / 2.0;
        self.lens_radius = self.aperture / 2.0;
    }

    pub fn get_ray(&self, s: Real, t: Real, rng: &mut Pcg32) -> Ray {
        let offset = if self.lens_radius > 0.0 {
            let (dx, dy) = concentric_disk(rng.next_real(), rng.next_real());
            self.lens_radius * (dx * self.u + dy * self.v)
        }
        else {
            Vec3::default()
        };

        let origin = self.position + offset;
        let target = self.upper_left_corner + s * self.horizontal + t * self.vertical;
        Ray::new(origin, target - origin, 0.0)
    }
}
