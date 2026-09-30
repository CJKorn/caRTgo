mod app;

use std::io;

use cartgo::color::Color;
use cartgo::geometry::{quad::Quad, sphere::Sphere, triangle::Triangle};
use cartgo::loader::obj::{Shading, UpAxis, load_obj};
use cartgo::material::principled::Principled;
use cartgo::math::{quat::Quat, vec3::Vec3};
use cartgo::render::camera::Camera;
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::light::{Light, PointLight, Sky, Sun};
use cartgo::render::render_settings::RenderSettings;
use cartgo::render::scene::Scene;

use crate::app::orbit::Orbit;

fn main() -> io::Result<()> {
    let spec = ImageSpec::new(640, 360);
    let mut scene = Scene::new();

    let ground = scene.add_material(Principled::default());
    let glass = scene.add_material(Principled {
        base_color: Color::WHITE,
        roughness: 0.0,
        transmission: 0.9,
        ..Default::default()
    });
    let red_plastic = scene.add_material(Principled {
        base_color: Color::new(0.8, 0.1, 0.1),
        roughness: 0.2,
        ..Default::default()
    });
    let gold = scene.add_material(Principled {
        base_color: Color::new(1.0, 0.77, 0.34),
        metallic: 1.0,
        roughness: 0.3,
        ..Default::default()
    });
    let argh_bright_light = scene.add_material(Principled {
        base_color: Color::new(1.0, 0.9, 0.7),
        emission: Color::new(1.0, 0.9, 0.7) * 10.0,
        ..Default::default()
    });

    // scene.add(Quad::new(
    //     Vec3::new(-50.0, -50.0, -0.5),
    //     Vec3::new(100.0, 0.0, 0.0),
    //     Vec3::new(0.0, 100.0, 0.0),
    //     ground,
    // ));
    scene.add(Sphere::new(Vec3::new(-1.1, 1.0, 0.0), 0.5, glass));
    load_obj("assets/models/BlenderTestExplort.obj", UpAxis::Y)?.add_to_scene(
        &mut scene,
        Shading::FromFile,
        red_plastic,
        Vec3::new(0.12, 1.0, -0.74),
        Quat::IDENTITY,
        Vec3::new(0.45, 0.45, 0.45),
    );
    // scene.add(Sphere::new(Vec3::new(1.1, 1.0, 0.0), 0.5, gold));
    // scene.add(Sphere::new(Vec3::new(0.0, 1.0, 1.5), 0.5, argh_bright_light));
    scene.set_sky(Sky {
        horizon: Color::new(0.35, 0.4, 0.45),
        zenith: Color::new(0.15, 0.25, 0.5),
    });
    scene.add_light(Light::Sun(Sun::new(Vec3::new(-1.0, -1.0, 0.8), Color::WHITE, 3.5, 6.0)));
    // scene.add_light(Light::Point(PointLight::new(
    //     Vec3::new(0.55, 0.6, 0.4),
    //     Color::new(1.0, 0.6, 0.3),
    //     0.3,
    //     0.05,
    // )));

    let camera = Camera::new(60.0, 1.0, 0.0, &spec);
    let orbit = Orbit::from_position(Vec3::new(0.0, -2.0, 0.6), Vec3::new(0.0, 1.0, 0.0));

    let settings = RenderSettings {
        samples_per_pixel: 1000,
        max_depth: 20,
        seed: 42,
        ..Default::default()
    };
    app::viewport::run(&spec, &settings, &scene, camera, orbit);
    Ok(())
}
