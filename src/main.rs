use std::io;

use cartgo::output::ppm::save_p3;
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::render::{gradient, render};

fn main() -> io::Result<()> {
    let spec = ImageSpec::new(256, 256);
    let pixels = render(&spec, gradient);
    save_p3("gradient.ppm", spec.width(), spec.height(), &pixels)?;
    Ok(())
}
