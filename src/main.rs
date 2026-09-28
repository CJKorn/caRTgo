use std::io;

use cartgo::geometry::{quad::Quad, sphere::Sphere};
use cartgo::math::vec3::Vec3;
use cartgo::output::ppm::save_p3;
use cartgo::render::camera::Camera;
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::render::{ray_color, render};
use cartgo::render::scene::Scene;

fn main() -> io::Result<()> {
    let spec = ImageSpec::new(400, 225);

    let sphere_center = Vec3::new(0.0, 1.0, 0.0);
    let mut scene = Scene::new();
    scene.add(Sphere::new(sphere_center, 0.5));
    scene.add(Quad::new(
        Vec3::new(-50.0, -50.0, -0.5),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(0.0, 100.0, 0.0),
    ));

    let mut camera = Camera::new(90.0, 1.0, 0.0, &spec);
    camera.look_at(sphere_center);

    let pixels = render(&spec, |s, t, rng| ray_color(&camera.get_ray(s, t, rng), &scene));
    save_p3("image.ppm", spec.width(), spec.height(), &pixels)?;
    Ok(())
}
