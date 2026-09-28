use std::sync::OnceLock;

use crate::accel::aabb::Aabb;
use crate::accel::bvh::{Bvh, BvhStats};
use crate::geometry::hittable::{HitRecord, Hittable};
use crate::geometry::triangle::moller_trumbore;
use crate::material::MaterialId;
use crate::math::{interval::Interval, ray::Ray, vec3::Vec3};

pub struct Mesh {
    vertices: Vec<Vec3>,
    triangles: Vec<[u32; 3]>,
    normals: Vec<Vec3>,
    normal_indices: Vec<Option<[u32; 3]>>,
    material: MaterialId,
    bvh: OnceLock<Bvh>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vec3>, triangles: Vec<[u32; 3]>, material: MaterialId) -> Self {
        Self::with_normals(vertices, triangles, Vec::new(), Vec::new(), material)
    }

    pub fn with_normals(
        vertices: Vec<Vec3>,
        triangles: Vec<[u32; 3]>,
        normals: Vec<Vec3>,
        normal_indices: Vec<Option<[u32; 3]>>,
        material: MaterialId,
    ) -> Self {
        let has_normals = !normal_indices.is_empty();
        assert!(!has_normals || normal_indices.len() == triangles.len());

        let mut kept_triangles = Vec::with_capacity(triangles.len());
        let mut kept_normals = Vec::with_capacity(normal_indices.len());
        for (i, &triangle) in triangles.iter().enumerate() {
            let [a, b, c] = triangle.map(|v| vertices[v as usize]);
            // Also catches NaN
            if !((b - a).cross(c - a).length_squared() > 0.0) {
                continue;
            }
            kept_triangles.push(triangle);
            if has_normals {
                kept_normals.push(normal_indices[i]);
            }
        }

        Self {
            vertices,
            triangles: kept_triangles,
            normals,
            normal_indices: kept_normals,
            material,
            bvh: OnceLock::new(),
        }
    }

    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    fn bvh(&self) -> &Bvh {
        self.bvh.get_or_init(|| {
            let boxes: Vec<Aabb> = self
                .triangles
                .iter()
                .map(|t| Aabb::from_points(&t.map(|v| self.vertices[v as usize])))
                .collect();
            Bvh::build(&boxes)
        })
    }

    pub fn bvh_stats(&self) -> BvhStats {
        self.bvh().stats()
    }
}

impl Hittable for Mesh {
    fn hit(&self, ray: &Ray, t_range: Interval) -> Option<HitRecord> {
        self.bvh().hit(ray, t_range, |i, range| {
            let [a, b, c] = self.triangles[i].map(|v| self.vertices[v as usize]);
            let edge1 = b - a;
            let edge2 = c - a;
            let (u, v, t) = moller_trumbore(ray, a, edge1, edge2)?;
            if u < 0.0 || v < 0.0 || u + v > 1.0 || !range.surrounds(t) {
                return None;
            }

            let geometric = edge1.cross(edge2).normalize();
            let normal = match self.normal_indices.get(i).copied().flatten() {
                Some([na, nb, nc]) => {
                    let n = (1.0 - u - v) * self.normals[na as usize]
                        + u * self.normals[nb as usize]
                        + v * self.normals[nc as usize];
                    if n.length_squared() > 0.0 {
                        let n = n.normalize();
                        if n.dot(geometric) < 0.0 {
                            -n
                        }
                        else {
                            n
                        }
                    }
                    else {
                        geometric
                    }
                }
                None => geometric,
            };
            Some(HitRecord::new(ray, t, normal, self.material))
        })
    }

    fn bounding_box(&self) -> Aabb {
        self.bvh().bounds()
    }
}
