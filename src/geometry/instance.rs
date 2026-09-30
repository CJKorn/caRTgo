use std::sync::Arc;

use crate::accel::aabb::Aabb;
use crate::geometry::hittable::{HitRecord, Hittable};
use crate::material::MaterialId;
use crate::math::{interval::Interval, quat::Quat, ray::Ray, vec3::Vec3};

pub struct Instance {
    object: Arc<dyn Hittable>,
    position: Vec3,
    rotation: Quat,
    inverse_rotation: Quat,
    inverse_scale: Vec3,
    material: Option<MaterialId>,
    bbox: Aabb,
}

impl Instance {
    // Scale, then rotate, then move, like Blender's object transform
    pub fn new(object: Arc<dyn Hittable>, position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        assert!(
            scale.x() != 0.0 && scale.y() != 0.0 && scale.z() != 0.0,
            "instance scale can't be zero"
        );
        let rotation = rotation.normalize();

        let local = object.bounding_box();
        let (x, y, z) = (local.axis(0), local.axis(1), local.axis(2));
        let corners = [0, 1, 2, 3, 4, 5, 6, 7].map(|i| {
            let corner = Vec3::new(
                if i & 1 == 0 { x.min } else { x.max },
                if i & 2 == 0 { y.min } else { y.max },
                if i & 4 == 0 { z.min } else { z.max },
            );
            rotation.rotate(corner.mul_elem(scale)) + position
        });

        Self {
            object,
            position,
            rotation,
            inverse_rotation: rotation.conjugate(),
            inverse_scale: scale.recip(),
            material: None,
            bbox: Aabb::from_points(&corners),
        }
    }

    pub fn with_material(mut self, material: MaterialId) -> Self {
        self.material = Some(material);
        self
    }
}

impl Hittable for Instance {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        // Scaling origin and direction by the same factor keeps t the same in both spaces
        let local_ray = Ray::new(
            self.inverse_rotation.rotate(ray.origin() - self.position).mul_elem(self.inverse_scale),
            self.inverse_rotation.rotate(ray.direction()).mul_elem(self.inverse_scale),
            ray.time(),
        );

        let mut hit = self.object.hit(&local_ray, t_range)?;
        hit.point = ray.at(hit.t);
        // Inverse transpose of rotate * scale
        hit.normal = self.rotation.rotate(hit.normal.mul_elem(self.inverse_scale)).normalize();
        if let Some(material) = self.material {
            hit.material = material;
        }
        Some(hit)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
