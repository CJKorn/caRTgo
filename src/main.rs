use std::io;

use cartgo::output::ppm::save_p3;
use cartgo::render::camera::Camera;
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::render::{SPHERE_CENTER, ray_color, render};

fn main() -> io::Result<()> {
    let spec = ImageSpec::new(400, 225);
    let mut camera = Camera::new(90.0, 1.0, 0.0, &spec);
    camera.look_at(SPHERE_CENTER);
    let pixels = render(&spec, |s, t, rng| ray_color(&camera.get_ray(s, t, rng)));
    save_p3("image.ppm", spec.width(), spec.height(), &pixels)?;
    Ok(())
}
