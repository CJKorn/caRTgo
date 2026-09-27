use std::io;

use cartgo::output::ppm::save_p3;
use cartgo::render::camera::{Camera, CameraDesc};
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::render::{ray_color, render};

fn main() -> io::Result<()> {
    let spec = ImageSpec::new(400, 225);
    let camera = Camera::new(&CameraDesc::default(), &spec);
    let pixels = render(&spec, |s, t, rng| ray_color(&camera.get_ray(s, t, rng)));
    save_p3("image.ppm", spec.width(), spec.height(), &pixels)?;
    Ok(())
}
