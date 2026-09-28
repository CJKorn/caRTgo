use crate::geometry::hittable::{HitRecord, Hittable};
use crate::math::{interval::Interval, ray::Ray};

#[derive(Default)]
pub struct Scene {
    objects: Vec<Box<dyn Hittable>>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, object: impl Hittable + 'static) {
        self.objects.push(Box::new(object));
    }

    pub fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        let mut closest: Option<HitRecord> = None;
        let mut range = t_range;
        for object in &self.objects {
            if let Some(hit) = object.hit(ray, range) {
                range.max = hit.t;
                closest = Some(hit);
            }
        }
        closest
    }
}
