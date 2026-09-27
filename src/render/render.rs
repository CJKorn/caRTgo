use crate::color::Color;
use crate::math::Real;
use crate::render::image_spec::ImageSpec;

pub fn render(spec: &ImageSpec, shade: impl Fn(Real, Real, &mut Pcg32) -> Color) -> Vec<Color>
    let mut pixels = Vec::with_capacity(spec.pixel_count());
    for y in 0..spec.height() {
        for x in 0..spec.width() {
            let u = x as Real * spec.inv_width();
            let v = y as Real * spec.inv_height();
            pixels.push(shade(u, v));
        }
    }
    pixels
}

//To remove
pub fn gradient(u: Real, v: Real) -> Color {
    Color::new(u, v, 0.25)
}
