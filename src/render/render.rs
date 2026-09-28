use crate::color::Color;
use crate::math::{EPS, Real, interval::Interval, ray::Ray};
use crate::render::image_spec::ImageSpec;
use crate::render::render_settings::RenderSettings;
use crate::render::scene::Scene;
use crate::rng::Pcg32;

const SKY_BLUE: Color = Color::new(0.5, 0.7, 1.0);

pub fn render(
    spec: &ImageSpec,
    settings: &RenderSettings,
    shade: impl Fn(Real, Real, &mut Pcg32) -> Color,
) -> Vec<Color> {
    let mut pixels = Vec::with_capacity(spec.pixel_count());
    for y in 0..spec.height() {
        for x in 0..spec.width() {
            let mut rng = pixel_rng(spec, settings.seed, x, y);

            let mut sum = Color::BLACK;
            for _ in 0..settings.samples_per_pixel {
                let s = (x as Real + rng.next_real()) * spec.inv_width();
                let t = (y as Real + rng.next_real()) * spec.inv_height();
                sum += shade(s, t, &mut rng);
            }
            pixels.push(sum / settings.samples_per_pixel as Real);
        }
    }
    pixels
}

fn pixel_rng(spec: &ImageSpec, seed: u64, x: u32, y: u32) -> Pcg32 {
    let index = y as u64 * spec.width() as u64 + x as u64;
    Pcg32::new(index.wrapping_mul(0x9E37_79B9_7F4A_7C15), seed)
}

pub fn ray_color(ray: &Ray, scene: &Scene, settings: &RenderSettings, rng: &mut Pcg32) -> Color {
    let mut ray = *ray;
    let mut throughput = Color::WHITE;
    let mut radiance = Color::BLACK;

    for _ in 0..settings.max_depth {
        let Some(hit) = scene.hit(&ray, Interval::new(EPS, Real::INFINITY)) else {
            return radiance + throughput * sky(&ray);
        };

        let material = scene.material(hit.material);
        radiance += throughput * material.emission;

        let Some(scatter) = material.scatter(&ray, &hit, rng) else {
            return radiance;
        };
        throughput *= scatter.attenuation;
        ray = scatter.ray;
    }
    radiance
}

// Should move to scene
fn sky(ray: &Ray) -> Color {
    let a = 0.5 * (ray.direction().normalize().z() + 1.0);
    Color::WHITE.lerp(SKY_BLUE, a)
}
