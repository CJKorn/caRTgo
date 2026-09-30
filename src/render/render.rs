use std::f32::consts::PI;
use std::sync::Mutex;
use std::thread;

use crate::color::Color;
use crate::geometry::hittable::HitRecord;
use crate::math::{EPS, Real, interval::Interval, ray::Ray};
use crate::render::image_spec::ImageSpec;
use crate::render::light::Light;
use crate::render::render_settings::RenderSettings;
use crate::render::scene::Scene;
use crate::rng::Pcg32;

pub fn render(
    spec: &ImageSpec,
    settings: &RenderSettings,
    shade: impl Fn(Real, Real, &mut Pcg32) -> Color + Sync,
    mut on_pass: impl FnMut(u32),
) -> Vec<Color> {
    let mut sums = vec![Color::BLACK; spec.pixel_count()];
    for sample in 0..settings.samples_per_pixel {
        render_pass(spec, settings.seed, sample, &mut sums, &shade, &|| false);
        on_pass(sample + 1);
    }
    let samples = settings.samples_per_pixel as Real;
    sums.into_iter().map(|sum| sum / samples).collect()
}

pub fn render_pass(
    spec: &ImageSpec,
    seed: u64,
    sample: u32,
    sums: &mut [Color],
    shade: &(impl Fn(Real, Real, &mut Pcg32) -> Color + Sync),
    cancel: &(impl Fn() -> bool + Sync),
) -> bool {
    let width = spec.width() as usize;
    let rows = Mutex::new(sums.chunks_mut(width).enumerate());
    let threads = thread::available_parallelism().map_or(1, |n| n.get());

    thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                loop {
                    if cancel() {
                        break;
                    }
                    let Some((y, row)) = rows.lock().unwrap().next() else {
                        break;
                    };
                    for (x, sum) in row.iter_mut().enumerate() {
                        let mut rng = sample_rng(seed, (y * width + x) as u64, sample);
                        let s = (x as Real + rng.next_real()) * spec.inv_width();
                        let t = (y as Real + rng.next_real()) * spec.inv_height();
                        *sum += shade(s, t, &mut rng);
                    }
                }
            });
        }
    });
    !cancel()
}

fn sample_rng(seed: u64, pixel: u64, sample: u32) -> Pcg32 {
    let key = pixel.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (sample as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    Pcg32::new(key, seed)
}

pub fn ray_color(ray: &Ray, scene: &Scene, settings: &RenderSettings, rng: &mut Pcg32) -> Color {
    let mut ray = *ray;
    let mut throughput = Color::WHITE;
    let mut radiance = Color::BLACK;
    let mut last_diffuse = false;

    for bounce in 0..settings.max_depth {
        let clamp = |c: Color| {
            if bounce == 0 {
                c
            }
            else {
                clamp_brightness(c, settings.clamp_indirect)
            }
        };

        let Some(hit) = scene.hit(&ray, Interval::new(EPS, Real::INFINITY)) else {
            let mut light = scene.sky().color(ray.direction());
            if !last_diffuse {
                for scene_light in scene.lights() {
                    light += scene_light.radiance(ray.direction());
                }
            }
            return radiance + clamp(throughput * light);
        };

        let material = scene.material(hit.material);
        radiance += clamp(throughput * material.emission);

        let albedo = material.diffuse_albedo(&ray, &hit);
        if albedo != Color::BLACK {
            for light in scene.lights() {
                radiance += clamp(throughput * albedo * direct_light(scene, light, &hit, ray.time(), rng));
            }
        }

        let Some(scatter) = material.scatter(&ray, &hit, rng) else {
            return radiance;
        };
        throughput *= scatter.attenuation;
        last_diffuse = scatter.diffuse;
        ray = scatter.ray;
    }
    radiance
}

fn clamp_brightness(c: Color, max: Real) -> Color {
    let peak = c.r().max(c.g()).max(c.b());
    if max > 0.0 && peak > max {
        c * (max / peak)
    }
    else {
        c
    }
}

fn direct_light(scene: &Scene, light: &Light, hit: &HitRecord, time: Real, rng: &mut Pcg32) -> Color {
    let Some(sample) = light.sample(hit.point, rng.next_real(), rng.next_real()) else {
        return Color::BLACK;
    };
    let cos = sample.direction.dot(hit.normal);
    if cos <= 0.0 {
        return Color::BLACK;
    }

    let shadow_ray = Ray::new(hit.point, sample.direction, time);
    if scene.hit(&shadow_ray, Interval::new(EPS, sample.distance - EPS)).is_some() {
        return Color::BLACK;
    }
    sample.irradiance * (cos / PI)
}
