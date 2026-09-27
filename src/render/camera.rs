use crate::math::{Real, ray::Ray, sampling::concentric_disk, vec3::Vec3};
use crate::render::image_spec::ImageSpec;
use crate::rng::Pcg32;

#[derive(Debug, Clone, Copy)]
pub struct CameraDesc {
    pub look_from: Vec3,
    pub look_at: Vec3,
    pub up: Vec3,
    pub vfov: Real,
    pub focus_dist: Real,
    pub aperture: Real,
}

impl Default for CameraDesc {
    // Pinhole camera
    fn default() -> Self {
        Self {
            look_from: Vec3::new(0.0, 0.0, 0.0),
            look_at: Vec3::new(0.0, 0.0, -1.0),
            up: Vec3::new(0.0, 1.0, 0.0),
            vfov: 90.0,
            focus_dist: 1.0,
            aperture: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    origin: Vec3,
    upper_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: Real,
}

impl Camera {
    pub fn new(desc: &CameraDesc, spec: &ImageSpec) -> Self {
        let theta = desc.vfov.to_radians();
        let viewport_height = 2.0 * (theta / 2.0).tan() * desc.focus_dist;
        let viewport_width = spec.aspect() * viewport_height;

        let w = (desc.look_from - desc.look_at).normalize();
        let u = desc.up.cross(w).normalize();
        let v = w.cross(u);
        debug_assert!(
            u.x().is_finite() && u.y().is_finite() && u.z().is_finite(),
            "camera basis is degenerate: look_from == look_at, or up is parallel to the view direction"
        );

        let origin = desc.look_from;
        let horizontal = viewport_width * u;
        let vertical = -viewport_height * v;
        let upper_left_corner = origin - desc.focus_dist * w - horizontal / 2.0 - vertical / 2.0;

        Self {
            origin,
            upper_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius: desc.aperture / 2.0,
        }
    }

    pub fn get_ray(&self, s: Real, t: Real, rng: &mut Pcg32) -> Ray {
        let offset = if self.lens_radius > 0.0 {
            let (dx, dy) = concentric_disk(rng.next_real(), rng.next_real());
            self.lens_radius * (dx * self.u + dy * self.v)
        }
        else {
            Vec3::default()
        };

        let origin = self.origin + offset;
        let target = self.upper_left_corner + s * self.horizontal + t * self.vertical;
        Ray::new(origin, target - origin, 0.0)
    }
}
