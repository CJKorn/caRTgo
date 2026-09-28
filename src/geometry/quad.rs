use crate::geometry::hittable::{HitRecord, Hittable};
use crate::geometry::triangle::moller_trumbore;
use crate::math::{interval::Interval, ray::Ray, vec3::Vec3};

// Parallelogram with corners corner, corner + u, corner + v, corner + u + v.
#[derive(Debug, Clone, Copy)]
pub struct Quad {
    corner: Vec3,
    u: Vec3,
    v: Vec3,
    normal: Vec3,
}

impl Quad {
    pub fn new(corner: Vec3, u: Vec3, v: Vec3) -> Self {
        Self {
            corner,
            u,
            v,
            normal: u.cross(v).normalize(),
        }
    }
}

impl Hittable for Quad {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        let (a, b, t) = moller_trumbore(ray, self.corner, self.u, self.v)?;
        if !(0.0..=1.0).contains(&a) || !(0.0..=1.0).contains(&b) || !t_range.surrounds(t) {
            return None;
        }
        Some(HitRecord::new(ray, t, self.normal))
    }
}
