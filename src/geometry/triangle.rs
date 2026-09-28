use crate::geometry::hittable::{HitRecord, Hittable};
use crate::math::{Real, interval::Interval, ray::Ray, vec3::Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    a: Vec3,
    edge1: Vec3,
    edge2: Vec3,
    normal: Vec3,
}

impl Triangle {
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let edge1 = b - a;
        let edge2 = c - a;
        Self {
            a,
            edge1,
            edge2,
            normal: edge1.cross(edge2).normalize(),
        }
    }
}

// Moller-Trumbore
impl Hittable for Triangle {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        let (u, v, t) = moller_trumbore(ray, self.a, self.edge1, self.edge2)?;
        if u < 0.0 || v < 0.0 || u + v > 1.0 || !t_range.surrounds(t) {
            return None;
        }
        Some(HitRecord::new(ray, t, self.normal))
    }
}

pub(crate) fn moller_trumbore(ray: &Ray, a: Vec3, edge1: Vec3, edge2: Vec3) -> Option<(Real, Real, Real)> {
    let p = ray.direction().cross(edge2);
    let det = edge1.dot(p);
    if det.abs() < Real::EPSILON {
        return None;
    }
    let inv_det = 1.0 / det;

    let s = ray.origin() - a;
    let u = s.dot(p) * inv_det;
    let q = s.cross(edge1);
    let v = ray.direction().dot(q) * inv_det;
    let t = edge2.dot(q) * inv_det;
    Some((u, v, t))
}
