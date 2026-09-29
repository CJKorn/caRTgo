use std::io;

use indicatif::{ProgressBar, ProgressStyle};
use minifb::{Key, Window, WindowOptions};

use cartgo::color::Color;
use cartgo::geometry::{quad::Quad, sphere::Sphere, triangle::Triangle};
use cartgo::loader::obj::{Shading, UpAxis, load_obj};
use cartgo::material::principled::Principled;
use cartgo::math::{Real, quat::Quat, vec3::Vec3};
use cartgo::output::ppm::save_p3;
use cartgo::render::camera::Camera;
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::light::{Light, PointLight, Sky, Sun};
use cartgo::render::render::{ray_color, render_pass};
use cartgo::render::render_settings::RenderSettings;
use cartgo::render::scene::Scene;

fn main() -> io::Result<()> {
    let spec = ImageSpec::new(1920, 1080);
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
    let argh_bright_light = scene.add_material(Principled {
        base_color: Color::new(1.0, 0.9, 0.7),
        emission: Color::new(1.0, 0.9, 0.7) * 10.0,
        ..Default::default()
    });

    scene.add(Quad::new(
        Vec3::new(-50.0, -50.0, -0.5),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(0.0, 100.0, 0.0),
        ground,
    ));
    scene.add(Sphere::new(Vec3::new(-1.1, 1.0, 0.0), 0.5, glass));
    let mut bunny = load_obj("assets/models/bunny.obj", UpAxis::NegZ)?;
    bunny.transform(Vec3::new(0.12, 1.0, -0.74), Quat::IDENTITY, 0.45);
    scene.add(bunny.into_mesh(red_plastic, Shading::Smooth));
    scene.add(Sphere::new(Vec3::new(1.1, 1.0, 0.0), 0.5, gold));
    scene.add(Sphere::new(Vec3::new(0.0, 1.0, 1.5), 0.5, argh_bright_light));
    scene.set_sky(Sky {
        horizon: Color::new(0.35, 0.4, 0.45),
        zenith: Color::new(0.15, 0.25, 0.5),
    });
    scene.add_light(Light::Sun(Sun::new(Vec3::new(-1.0, -1.0, 0.8), Color::WHITE, 3.5, 6.0)));
    // scene.add_light(Light::Point(PointLight::new(
    //     Vec3::new(0.55, 0.6, 0.4),
    //     Color::new(1.0, 0.6, 0.3),
    //     0.3,
    //     0.05,
    // )));

    let mut camera = Camera::new(60.0, 1.0, 0.0, &spec);
    camera.set_position(Vec3::new(0.0, -2.0, 0.6));
    camera.look_at(Vec3::new(0.0, 1.0, 0.0));

    let settings = RenderSettings {
        samples_per_pixel: 100,
        max_depth: 20,
        seed: 42,
        ..Default::default()
    };
    let progress = ProgressBar::new(settings.samples_per_pixel as u64).with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {wide_bar} {pos}/{len} passes ({per_sec}, ETA {eta})")
            .unwrap(),
    );
    let shade = |s, t, rng: &mut _| ray_color(&camera.get_ray(s, t, rng), &scene, &settings, rng);

    let (width, height) = (spec.width() as usize, spec.height() as usize);
    let mut window = Window::new("caRTgo", width, height, WindowOptions::default())
        .unwrap_or_else(|e| panic!("could not open window: {e}"));
    window.set_target_fps(60);

    let mut sums = vec![Color::BLACK; spec.pixel_count()];
    let mut buffer = vec![0u32; spec.pixel_count()];
    let mut passes = 0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if passes < settings.samples_per_pixel {
            render_pass(&spec, settings.seed, passes, &mut sums, &shade);
            passes += 1;
            progress.inc(1);

            let average = |sum: Color| sum / passes as Real;
            for (pixel, &sum) in buffer.iter_mut().zip(&sums) {
                let [r, g, b] = average(sum).to_rgb8();
                *pixel = (r as u32) << 16 | (g as u32) << 8 | b as u32;
            }

            if passes == settings.samples_per_pixel {
                progress.finish();
                let pixels: Vec<Color> = sums.iter().map(|&sum| average(sum)).collect();
                save_p3("image.ppm", spec.width(), spec.height(), &pixels)?;
            }
        }
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
    Ok(())
}
