use crate::geometry::hittable::{HitRecord, Hittable};
use crate::math::{Real, interval::Interval, ray::Ray, vec3::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    center: Vec3,
    radius: Real,
}

impl Sphere {
    pub fn new(center: Vec3, radius: Real) -> Self {
        Self { center, radius }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        let oc = self.center - ray.origin();
        let a = ray.direction().length_squared();
        let h = ray.direction().dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let mut t = (h - sqrt_d) / a;
        if !t_range.surrounds(t) {
            t = (h + sqrt_d) / a;
            if !t_range.surrounds(t) {
                return None;
            }
        }

        let outward_normal = (ray.at(t) - self.center) / self.radius;
        Some(HitRecord::new(ray, t, outward_normal))
    }
}
