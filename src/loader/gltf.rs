use std::f32::consts::FRAC_PI_2;
use std::io;
use std::path::Path;
use std::sync::Arc;

use ::gltf::camera::Projection;
use ::gltf::khr_lights_punctual::Kind;
use ::gltf::mesh::Mode;

use crate::color::Color;
use crate::geometry::instance::Instance;
use crate::geometry::mesh::Mesh;
use crate::material::principled::Principled;
use crate::math::{Real, quat::Quat, vec3::Vec3};
use crate::render::light::{Light, PointLight, Sun};
use crate::render::scene::{Scene, SceneCamera};

// Blender's glTF exporter converts watts to lumens at 683 lm/W
const LUMENS_PER_WATT: Real = 683.0;

#[derive(Debug, Clone, Copy)]
struct Transform {
    position: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Transform {
    // Exact unless a parent has non-uniform scale and the child is rotated (that would need shear)
    fn then(self, child: Transform) -> Transform {
        Transform {
            position: self.position + self.rotation.rotate(child.position.mul_elem(self.scale)),
            rotation: self.rotation * child.rotation,
            scale: self.scale.mul_elem(child.scale),
        }
    }
}

pub fn load_gltf(path: impl AsRef<Path>, scene: &mut Scene) -> io::Result<()> {
    let (document, buffers, _) =
        ::gltf::import(path).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let default_material = scene.add_material(Principled::default());
    let materials: Vec<_> = document
        .materials()
        .map(|m| {
            let pbr = m.pbr_metallic_roughness();
            let [r, g, b, _] = pbr.base_color_factor();
            let [er, eg, eb] = m.emissive_factor();
            scene.add_material(Principled {
                base_color: Color::new(r, g, b),
                metallic: pbr.metallic_factor(),
                roughness: pbr.roughness_factor(),
                ior: m.ior().unwrap_or(1.5),
                transmission: m.transmission().map_or(0.0, |t| t.transmission_factor()),
                emission: Color::new(er, eg, eb) * m.emissive_strength().unwrap_or(1.0),
            })
        })
        .collect();

    let meshes: Vec<Vec<Arc<Mesh>>> = document
        .meshes()
        .map(|mesh| {
            mesh.primitives()
                .filter(|p| p.mode() == Mode::Triangles)
                .filter_map(|primitive| {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                    let vertices: Vec<Vec3> = reader.read_positions()?.map(to_vec3).collect();
                    let indices: Vec<u32> = match reader.read_indices() {
                        Some(indices) => indices.into_u32().collect(),
                        None => (0..vertices.len() as u32).collect(),
                    };
                    let triangles: Vec<[u32; 3]> =
                        indices.chunks_exact(3).map(|t| [t[0], t[1], t[2]]).collect();
                    let material = primitive.material().index().map_or(default_material, |i| materials[i]);

                    let mesh = match reader.read_normals() {
                        Some(normals) => {
                            let normals = normals.map(to_vec3).collect();
                            let normal_indices = triangles.iter().map(|&t| Some(t)).collect();
                            Mesh::with_normals(vertices, triangles, normals, normal_indices, material)
                        }
                        None => Mesh::new(vertices, triangles, material),
                    };
                    Some(Arc::new(mesh))
                })
                .collect()
        })
        .collect();

    // glTF is Y-up
    let root = Transform {
        position: Vec3::default(),
        rotation: Quat::from_axis_angle(Vec3::X, FRAC_PI_2),
        scale: Vec3::new(1.0, 1.0, 1.0),
    };
    let gltf_scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "glTF file has no scenes"))?;
    for node in gltf_scene.nodes() {
        add_node(&node, root, &meshes, scene);
    }
    Ok(())
}

fn add_node(node: &::gltf::Node, parent: Transform, meshes: &[Vec<Arc<Mesh>>], scene: &mut Scene) {
    let (position, [x, y, z, w], scale) = node.transform().decomposed();
    let world = parent.then(Transform {
        position: to_vec3(position),
        rotation: Quat::new(w, x, y, z),
        scale: to_vec3(scale),
    });

    if let Some(mesh) = node.mesh() {
        for primitive in &meshes[mesh.index()] {
            scene.add(Instance::new(primitive.clone(), world.position, world.rotation, world.scale));
        }
    }

    if let Some(camera) = node.camera() {
        if let Projection::Perspective(perspective) = camera.projection() {
            scene.add_camera(SceneCamera {
                name: node.name().or(camera.name()).unwrap_or("Camera").to_string(),
                position: world.position,
                rotation: world.rotation,
                vfov: perspective.yfov().to_degrees(),
            });
        }
    }

    if let Some(light) = node.light() {
        let [r, g, b] = light.color();
        let color = Color::new(r, g, b);
        let strength = light.intensity() / LUMENS_PER_WATT;
        match light.kind() {
            // Lights shine down their local -Z, so +Z points back at a sun
            Kind::Directional => {
                let direction = world.rotation.rotate(Vec3::Z);
                scene.add_light(Light::Sun(Sun::new(direction, color, strength, 0.5)));
            }
            Kind::Point => {
                scene.add_light(Light::Point(PointLight::new(world.position, color, strength, 0.0)));
            }
            Kind::Spot { .. } => eprintln!("skipping spot light {:?}, not supported", node.name()),
        }
    }

    for child in node.children() {
        add_node(&child, world, meshes, scene);
    }
}

fn to_vec3([x, y, z]: [f32; 3]) -> Vec3 {
    Vec3::new(x, y, z)
}
