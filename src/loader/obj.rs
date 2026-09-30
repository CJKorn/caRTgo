use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;

use crate::color::Color;
use crate::geometry::instance::Instance;
use crate::geometry::mesh::Mesh;
use crate::material::MaterialId;
use crate::material::principled::Principled;
use crate::math::{Real, quat::Quat, vec3::Vec3};
use crate::render::scene::Scene;

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

pub struct ObjObject {
    pub name: String,
    // Index into ObjFile::materials
    pub material: Option<usize>,
    pub data: ObjData,
}

pub struct ObjFile {
    pub objects: Vec<ObjObject>,
    pub materials: Vec<Principled>,
}

impl ObjData {
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

impl ObjFile {
    // Objects without a material from the .mtl use default_material
    pub fn add_to_scene(
        self,
        scene: &mut Scene,
        shading: Shading,
        default_material: MaterialId,
        position: Vec3,
        rotation: Quat,
        scale: Vec3,
    ) {
        let ids: Vec<MaterialId> = self.materials.into_iter().map(|m| scene.add_material(m)).collect();
        for object in self.objects {
            let material = object.material.map_or(default_material, |i| ids[i]);
            let mesh = Arc::new(object.data.into_mesh(material, shading));
            scene.add(Instance::new(mesh, position, rotation, scale));
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

struct Group {
    name: String,
    material: Option<usize>,
    triangles: Vec<[u32; 3]>,
    normal_indices: Vec<Option<[u32; 3]>>,
}

impl Group {
    fn new(name: String, material: Option<usize>) -> Self {
        Self {
            name,
            material,
            triangles: Vec::new(),
            normal_indices: Vec::new(),
        }
    }
}

pub fn load_obj(path: impl AsRef<Path>, up: UpAxis) -> io::Result<ObjFile> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)?;
    let convert = |x: Real, y: Real, z: Real| match up {
        UpAxis::Y => Vec3::new(x, -z, y),
        UpAxis::NegY => Vec3::new(x, z, -y),
        UpAxis::Z => Vec3::new(x, y, z),
        UpAxis::NegZ => Vec3::new(x, -y, -z),
    };

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut materials = Vec::new();
    let mut material_names: HashMap<String, usize> = HashMap::new();
    let mut groups = vec![Group::new(String::new(), None)];

    for (line_index, line) in text.lines().enumerate() {
        let error = |message: &str| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}:{}: {message}", path.display(), line_index + 1),
            )
        };

        let mut tokens = line.split_whitespace();
        let keyword = tokens.next();
        let rest = tokens.clone().collect::<Vec<_>>().join(" ");
        let mut coord = || -> io::Result<Real> {
            tokens
                .next()
                .and_then(|t| t.parse().ok())
                .ok_or_else(|| error("expected 3 numbers"))
        };
        let group = groups.last_mut().unwrap();

        match keyword {
            Some("v") => {
                let (x, y, z) = (coord()?, coord()?, coord()?);
                vertices.push(convert(x, y, z));
            }
            Some("vn") => {
                let (x, y, z) = (coord()?, coord()?, coord()?);
                normals.push(convert(x, y, z));
            }
            Some("o") => {
                let material = group.material;
                groups.push(Group::new(rest, material));
            }
            Some("usemtl") => {
                let material = material_names.get(&rest).copied();
                if group.triangles.is_empty() {
                    group.material = material;
                }
                else {
                    let name = group.name.clone();
                    groups.push(Group::new(name, material));
                }
            }
            Some("mtllib") => {
                let mtl_path = path.parent().unwrap_or(Path::new("")).join(&rest);
                for (name, material) in load_mtl(&mtl_path)? {
                    material_names.insert(name, materials.len());
                    materials.push(material);
                }
            }
            Some("f") => {
                let corners = tokens
                    .map(|t| {
                        face_corner(t, vertices.len(), normals.len())
                            .ok_or_else(|| error(&format!("bad face vertex {t}")))
                    })
                    .collect::<io::Result<Vec<(u32, Option<u32>)>>>()?;
                if corners.len() < 3 {
                    return Err(error("face needs at least 3 vertices"));
                }

                let all_normals = corners.iter().all(|(_, n)| n.is_some());
                for k in 1..corners.len() - 1 {
                    let [a, b, c] = [corners[0], corners[k], corners[k + 1]];
                    group.triangles.push([a.0, b.0, c.0]);
                    group.normal_indices.push(all_normals.then(|| [a.1, b.1, c.1].map(Option::unwrap)));
                }
            }
            _ => {}
        }
    }

    let objects = groups
        .into_iter()
        .filter(|g| !g.triangles.is_empty())
        .map(|g| compact(g, &vertices, &normals))
        .collect();
    Ok(ObjFile { objects, materials })
}

// Indices in an OBJ are file-wide
fn compact(group: Group, vertices: &[Vec3], normals: &[Vec3]) -> ObjObject {
    let mut data = ObjData {
        vertices: Vec::new(),
        triangles: Vec::with_capacity(group.triangles.len()),
        normals: Vec::new(),
        normal_indices: Vec::with_capacity(group.normal_indices.len()),
    };
    let (mut vertex_map, mut normal_map) = (HashMap::new(), HashMap::new());

    for (triangle, normal) in group.triangles.iter().zip(&group.normal_indices) {
        data.triangles.push(triangle.map(|v| remap(v, &mut vertex_map, vertices, &mut data.vertices)));
        data.normal_indices.push(normal.map(|n| n.map(|n| remap(n, &mut normal_map, normals, &mut data.normals))));
    }

    ObjObject {
        name: group.name,
        material: group.material,
        data,
    }
}

fn remap(index: u32, map: &mut HashMap<u32, u32>, source: &[Vec3], out: &mut Vec<Vec3>) -> u32 {
    *map.entry(index).or_insert_with(|| {
        out.push(source[index as usize]);
        (out.len() - 1) as u32
    })
}

// PBR extension keys as written by Blender.
fn load_mtl(path: &Path) -> io::Result<Vec<(String, Principled)>> {
    let text = fs::read_to_string(path)?;
    let mut materials: Vec<(String, Principled)> = Vec::new();

    for (line_index, line) in text.lines().enumerate() {
        let error = || {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}:{}: bad number", path.display(), line_index + 1),
            )
        };
        let mut tokens = line.split_whitespace();
        let keyword = tokens.next();
        let numbers = tokens
            .clone()
            .map(|t| t.parse::<Real>())
            .collect::<Result<Vec<_>, _>>();

        if keyword == Some("newmtl") {
            materials.push((tokens.collect::<Vec<_>>().join(" "), Principled::default()));
            continue;
        }
        let Some((_, material)) = materials.last_mut() else {
            continue;
        };
        let value = || numbers.as_ref().ok().and_then(|n| n.first().copied()).ok_or_else(error);
        let color = || match numbers.as_deref() {
            Ok([r, g, b, ..]) => Ok(Color::new(*r, *g, *b)),
            _ => Err(error()),
        };
        // Wow ok this just works
        match keyword {
            Some("Kd") => material.base_color = color()?,
            Some("Ke") => material.emission = color()?,
            Some("Ni") => material.ior = value()?,
            Some("Pr") => material.roughness = value()?,
            Some("Pm") => material.metallic = value()?,
            Some("Tf") => {
                let tf = color()?;
                material.transmission = (tf.r() + tf.g() + tf.b()) / 3.0;
            }
            _ => {}
        }
    }
    Ok(materials)
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
