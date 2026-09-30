use std::sync::OnceLock;

use crate::accel::bvh::{Bvh, BvhStats};
use crate::geometry::hittable::{HitRecord, Hittable};
use crate::material::{MaterialId, principled::Principled};
use crate::math::{Real, interval::Interval, quat::Quat, ray::Ray, vec3::Vec3};
use crate::render::camera::Camera;
use crate::render::image_spec::ImageSpec;
use crate::render::light::{Light, Sky};

#[derive(Debug, Clone)]
pub struct SceneCamera {
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub vfov: Real,
}

impl SceneCamera {
    pub fn to_camera(&self, spec: &ImageSpec) -> Camera {
        let mut camera = Camera::new(self.vfov, 1.0, 0.0, spec);
        camera.set_position(self.position);
        camera.set_rotation(self.rotation);
        camera
    }
}

#[derive(Default)]
pub struct Scene {
    objects: Vec<Box<dyn Hittable>>,
    materials: Vec<Principled>,
    sky: Sky,
    lights: Vec<Light>,
    cameras: Vec<SceneCamera>,
    bvh: OnceLock<Bvh>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, object: impl Hittable + 'static) {
        self.objects.push(Box::new(object));
        self.bvh = OnceLock::new();
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

    pub fn add_camera(&mut self, camera: SceneCamera) {
        self.cameras.push(camera);
    }

    pub fn cameras(&self) -> &[SceneCamera] {
        &self.cameras
    }

    fn bvh(&self) -> &Bvh {
        self.bvh.get_or_init(|| {
            let boxes: Vec<_> = self.objects.iter().map(|o| o.bounding_box()).collect();
            Bvh::build(&boxes)
        })
    }

    pub fn bvh_stats(&self) -> BvhStats {
        self.bvh().stats()
    }

    pub fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        self.bvh().hit(ray, t_range, |i, range| {
            let mut hit = self.objects[i].hit(ray, range)?;
            hit.object = i as u32;
            Some(hit)
        })
    }

    pub fn occluded(&self, ray: &Ray, t_range: Interval) -> bool {
        self.bvh().occluded(ray, t_range, |i, range| self.objects[i].occluded(ray, range))
    }

    pub fn hit_linear(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        let mut closest: Option<HitRecord> = None;
        let mut range = t_range;
        for (i, object) in self.objects.iter().enumerate() {
            if let Some(mut hit) = object.hit(ray, range) {
                range.max = hit.t;
                hit.object = i as u32;
                closest = Some(hit);
            }
        }
        closest
    }
}
