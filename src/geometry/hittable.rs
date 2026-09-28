use crate::material::MaterialId;
use crate::math::{Real, interval::Interval, ray::Ray, vec3::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: Real,
    pub front_face: bool,
    pub material: MaterialId,
}

impl HitRecord {
    pub fn new(ray: &Ray, t: Real, outward_normal: Vec3, material: MaterialId) -> Self {
        let front_face = ray.direction().dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        }
        else {
            -outward_normal
        };

        Self {
            point: ray.at(t),
            normal,
            t,
            front_face,
            material,
        }
    }
}

// Google Trait
// Holy hell
pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord>;
}
