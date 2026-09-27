use crate::color::Color;
use crate::math::{Real, ray::Ray, vec3::Vec3};
use crate::render::image_spec::ImageSpec;
use crate::rng::Pcg32;

const SKY_BLUE: Color = Color::new(0.5, 0.7, 1.0);
const SPHERE_CENTER: Vec3 = Vec3::new(0.0, 0.0, -1.0);
const SPHERE_RADIUS: Real = 0.5;

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
    if let Some(t) = hit_sphere(SPHERE_CENTER, SPHERE_RADIUS, ray) {
        let n = (ray.at(t) - SPHERE_CENTER) / SPHERE_RADIUS;
        return 0.5 * Color::new(n.x() + 1.0, n.y() + 1.0, n.z() + 1.0);
    }

    let a = 0.5 * (ray.direction().normalize().y() + 1.0);
    Color::WHITE.lerp(SKY_BLUE, a)
}

fn hit_sphere(center: Vec3, radius: Real, ray: &Ray) -> Option<Real> {
    let oc = center - ray.origin();
    let a = ray.direction().length_squared();
    let h = ray.direction().dot(oc);
    let c = oc.length_squared() - radius * radius;

    let discriminant = h * h - a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_d = discriminant.sqrt();
    let near = (h - sqrt_d) / a;
    if near > 0.0 {
        return Some(near);
    }
    let far = (h + sqrt_d) / a;
    (far > 0.0).then_some(far)
}
