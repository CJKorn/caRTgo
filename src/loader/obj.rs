use std::fs;
use std::io;
use std::path::Path;

use crate::geometry::mesh::Mesh;
use crate::material::MaterialId;
use crate::math::{Real, quat::Quat, vec3::Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpAxis {
    Y,
    NegY,
    Z,
    NegZ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shading {
    FromFile,
    Flat,
    Smooth,
}

pub struct ObjData {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
    pub normals: Vec<Vec3>,
    pub normal_indices: Vec<Option<[u32; 3]>>,
}

impl ObjData {
    // Scale, then rotate, then move, like Blender's object transform
    pub fn transform(&mut self, position: Vec3, rotation: Quat, scale: Real) {
        for v in &mut self.vertices {
            *v = rotation.rotate(*v * scale) + position;
        }
        for n in &mut self.normals {
            *n = rotation.rotate(*n);
        }
    }

    pub fn into_mesh(self, material: MaterialId, shading: Shading) -> Mesh {
        let has_normals = self.normal_indices.iter().any(Option::is_some);
        match shading {
            Shading::FromFile if has_normals => {
                Mesh::with_normals(self.vertices, self.triangles, self.normals, self.normal_indices, material)
            }
            Shading::FromFile | Shading::Flat => Mesh::new(self.vertices, self.triangles, material),
            Shading::Smooth => {
                let normals = vertex_normals(&self.vertices, &self.triangles);
                let normal_indices = self.triangles.iter().map(|&t| Some(t)).collect();
                Mesh::with_normals(self.vertices, self.triangles, normals, normal_indices, material)
            }
        }
    }
}

fn vertex_normals(vertices: &[Vec3], triangles: &[[u32; 3]]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::default(); vertices.len()];
    for triangle in triangles {
        let [a, b, c] = triangle.map(|v| vertices[v as usize]);
        let face = (b - a).cross(c - a);
        for &v in triangle {
            normals[v as usize] += face;
        }
    }
    for n in &mut normals {
        if n.length_squared() > 0.0 {
            *n = n.normalize();
        }
    }
    normals
}

pub fn load_obj(path: impl AsRef<Path>, up: UpAxis) -> io::Result<ObjData> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)?;
    let mut data = ObjData {
        vertices: Vec::new(),
        triangles: Vec::new(),
        normals: Vec::new(),
        normal_indices: Vec::new(),
    };
    let convert = |x: Real, y: Real, z: Real| match up {
        UpAxis::Y => Vec3::new(x, -z, y),
        UpAxis::NegY => Vec3::new(x, z, -y),
        UpAxis::Z => Vec3::new(x, y, z),
        UpAxis::NegZ => Vec3::new(x, -y, -z),
    };

    for (line_index, line) in text.lines().enumerate() {
        let error = |message: &str| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}:{}: {message}", path.display(), line_index + 1),
            )
        };

        let mut tokens = line.split_whitespace();
        let keyword = tokens.next();
        let mut coord = || -> io::Result<Real> {
            tokens
                .next()
                .and_then(|t| t.parse().ok())
                .ok_or_else(|| error("expected 3 numbers"))
        };

        match keyword {
            Some("v") => {
                let (x, y, z) = (coord()?, coord()?, coord()?);
                data.vertices.push(convert(x, y, z));
            }
            Some("vn") => {
                let (x, y, z) = (coord()?, coord()?, coord()?);
                data.normals.push(convert(x, y, z));
            }
            Some("f") => {
                let (vertex_count, normal_count) = (data.vertices.len(), data.normals.len());
                let corners = tokens
                    .map(|t| {
                        face_corner(t, vertex_count, normal_count)
                            .ok_or_else(|| error(&format!("bad face vertex {t}")))
                    })
                    .collect::<io::Result<Vec<(u32, Option<u32>)>>>()?;
                if corners.len() < 3 {
                    return Err(error("face needs at least 3 vertices"));
                }

                let all_normals = corners.iter().all(|(_, n)| n.is_some());
                for k in 1..corners.len() - 1 {
                    let [a, b, c] = [corners[0], corners[k], corners[k + 1]];
                    data.triangles.push([a.0, b.0, c.0]);
                    data.normal_indices.push(all_normals.then(|| [a.1, b.1, c.1].map(Option::unwrap)));
                }
            }
            _ => {}
        }
    }
    Ok(data)
}

fn face_corner(token: &str, vertex_count: usize, normal_count: usize) -> Option<(u32, Option<u32>)> {
    let mut parts = token.split('/');
    let vertex = resolve_index(parts.next()?, vertex_count)?;
    let _texture = parts.next();
    let normal = match parts.next() {
        Some(n) if !n.is_empty() => Some(resolve_index(n, normal_count)?),
        _ => None,
    };
    Some((vertex, normal))
}

fn resolve_index(token: &str, count: usize) -> Option<u32> {
    let index: i64 = token.parse().ok()?;
    let resolved = match index {
        i if i > 0 => i - 1,
        i if i < 0 => count as i64 + i,
        _ => return None,
    };
    (0..count as i64).contains(&resolved).then_some(resolved as u32)
}
