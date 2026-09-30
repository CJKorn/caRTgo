use std::sync::Arc;

use crate::accel::aabb::Aabb;
use crate::geometry::hittable::{HitRecord, Hittable};
use crate::math::{Real, interval::Interval, quat::Quat, ray::Ray, vec3::Vec3};

pub struct Instance {
    object: Arc<dyn Hittable>,
    position: Vec3,
    rotation: Quat,
    scale: Real,
    bbox: Aabb,
}

impl Instance {
    // Scale, then rotate, then move, like Blender's object transform
    pub fn new(object: Arc<dyn Hittable>, position: Vec3, rotation: Quat, scale: Real) -> Self {
        assert!(scale > 0.0, "instance scale must be positive");
        let rotation = rotation.normalize();

        let local = object.bounding_box();
        let (x, y, z) = (local.axis(0), local.axis(1), local.axis(2));
        let corners = [0, 1, 2, 3, 4, 5, 6, 7].map(|i| {
            let corner = Vec3::new(
                if i & 1 == 0 { x.min } else { x.max },
                if i & 2 == 0 { y.min } else { y.max },
                if i & 4 == 0 { z.min } else { z.max },
            );
            rotation.rotate(corner * scale) + position
        });

        Self {
            object,
            position,
            rotation,
            scale,
            bbox: Aabb::from_points(&corners),
        }
    }
}

impl Hittable for Instance {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        // Scaling origin and direction by the same factor keeps t the same in both spaces
        let inverse = self.rotation.conjugate();
        let local_ray = Ray::new(
            inverse.rotate(ray.origin() - self.position) / self.scale,
            inverse.rotate(ray.direction()) / self.scale,
            ray.time(),
        );

        let mut hit = self.object.hit(&local_ray, t_range)?;
        hit.point = ray.at(hit.t);
        // Uniform scale :(
        hit.normal = self.rotation.rotate(hit.normal);
        Some(hit)
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
