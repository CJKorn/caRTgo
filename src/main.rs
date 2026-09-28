use std::io;

use cartgo::color::Color;
use cartgo::geometry::{quad::Quad, sphere::Sphere, triangle::Triangle};
use cartgo::material::principled::Principled;
use cartgo::math::vec3::Vec3;
use cartgo::output::ppm::save_p3;
use cartgo::render::camera::Camera;
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::light::{Sky, Sun};
use cartgo::render::render::{ray_color, render};
use cartgo::render::render_settings::RenderSettings;
use cartgo::render::scene::Scene;

fn main() -> io::Result<()> {
    let spec = ImageSpec::new(500, 300);
    let mut scene = Scene::new();

    let ground = scene.add_material(Principled::default());
    let glass = scene.add_material(Principled {
        base_color: Color::WHITE,
        roughness: 0.0,
        transmission: 1.0,
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

    scene.add(Quad::new(
        Vec3::new(-50.0, -50.0, -0.5),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(0.0, 100.0, 0.0),
        ground,
    ));
    scene.add(Sphere::new(Vec3::new(-1.1, 1.0, 0.0), 0.5, glass));
    scene.add(Sphere::new(Vec3::new(0.0, 1.0, 0.0), 0.5, red_plastic));
    scene.add(Sphere::new(Vec3::new(1.1, 1.0, 0.0), 0.5, gold));
    scene.add(Triangle::new(
        Vec3::new(-0.5, 0.5, 1.0),
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(0.0, 1.5, 0.5),
        red_plastic,
    ));

    scene.set_sky(Sky {
        horizon: Color::new(0.35, 0.4, 0.45),
        zenith: Color::new(0.15, 0.25, 0.5),
    });
    scene.set_sun(Some(Sun::new(Vec3::new(-1.0, -1.0, 0.8), Color::WHITE, 3.5, 6.0)));

    let mut camera = Camera::new(60.0, 1.0, 0.0, &spec);
    camera.set_position(Vec3::new(0.0, -2.0, 0.6));
    camera.look_at(Vec3::new(0.0, 1.0, 0.0));

    let settings = RenderSettings {
        samples_per_pixel: 100,
        max_depth: 20,
        seed: 42,
        ..Default::default()
    };
    let pixels = render(&spec, &settings, |s, t, rng| {
        ray_color(&camera.get_ray(s, t, rng), &scene, &settings, rng)
    });
    save_p3("image.ppm", spec.width(), spec.height(), &pixels)?;
    Ok(())
}
