use crate::color::Color;
use crate::math::{Real, ray::Ray};
use crate::render::image_spec::ImageSpec;
use crate::rng::Pcg32;

const SKY_BLUE: Color = Color::new(0.5, 0.7, 1.0);

pub fn render(spec: &ImageSpec, shade: impl Fn(Real, Real, &mut Pcg32) -> Color) -> Vec<Color> {
    let mut pixels = Vec::with_capacity(spec.pixel_count());
    for y in 0..spec.height() {
        for x in 0..spec.width() {
            let mut rng = pixel_rng(spec, x, y);
            let s = (x as Real + 0.5) * spec.inv_width();
            let t = (y as Real + 0.5) * spec.inv_height();

            pixels.push(shade(s, t, &mut rng));
        }
    }
    pixels
}

fn pixel_rng(spec: &ImageSpec, x: u32, y: u32) -> Pcg32 {
    let index = y as u64 * spec.width() as u64 + x as u64;
    Pcg32::new(index.wrapping_mul(0x9E37_79B9_7F4A_7C15), 0)
}

pub fn ray_color(ray: &Ray) -> Color {
    let a = 0.5 * (ray.direction().normalize().y() + 1.0);
    Color::WHITE.lerp(SKY_BLUE, a)
}
