mod app;

use std::io;

use cartgo::color::Color;
use cartgo::loader::gltf::load_gltf;
use cartgo::math::{EPS, Real, interval::Interval, ray::Ray, vec3::Vec3};
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::light::Sky;
use cartgo::render::render_settings::RenderSettings;
use cartgo::render::scene::Scene;

use crate::app::orbit::Orbit;

fn main() -> io::Result<()> {
    let spec = ImageSpec::new(640, 360);
    let mut scene = Scene::new();

    load_gltf("assets/scenes/CornellBox.glb", &mut scene)?;
    scene.set_sky(Sky {
        horizon: Color::new(0.35, 0.4, 0.45),
        zenith: Color::new(0.15, 0.25, 0.5),
    });

    let scene_camera = scene
        .cameras()
        .first()
        .cloned()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "scene has no camera"))?;
    let camera = scene_camera.to_camera(&spec);

    // Orbit around whatever the camera is looking at
    let forward = scene_camera.rotation.rotate(-Vec3::Z);
    let view_ray = Ray::new(scene_camera.position, forward, 0.0);
    let distance = scene
        .hit(&view_ray, Interval::new(EPS, Real::INFINITY))
        .map_or(10.0, |hit| hit.t);
    let orbit = Orbit::from_position(scene_camera.position, scene_camera.position + distance * forward);

    let settings = RenderSettings {
        samples_per_pixel: 1000,
        max_depth: 20,
        seed: 42,
        ..Default::default()
    };
    app::viewport::run(&spec, &settings, &scene, camera, orbit);
    Ok(())
}
