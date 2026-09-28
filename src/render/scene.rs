use crate::geometry::hittable::{HitRecord, Hittable};
use crate::material::{MaterialId, principled::Principled};
use crate::math::{interval::Interval, ray::Ray};
use crate::render::light::{Light, Sky};

#[derive(Default)]
pub struct Scene {
    objects: Vec<Box<dyn Hittable>>,
    materials: Vec<Principled>,
    sky: Sky,
    lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, object: impl Hittable + 'static) {
        self.objects.push(Box::new(object));
    }

    pub fn add_material(&mut self, material: Principled) -> MaterialId {
        self.materials.push(material);
        (self.materials.len() - 1) as MaterialId
    }

    pub fn material(&self, id: MaterialId) -> &Principled {
        &self.materials[id as usize]
    }

    pub fn sky(&self) -> &Sky {
        &self.sky
    }

    pub fn set_sky(&mut self, sky: Sky) {
        self.sky = sky;
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn lights(&self) -> &[Light] {
        &self.lights
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
