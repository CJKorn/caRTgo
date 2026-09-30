use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressStyle};
use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};

use cartgo::color::Color;
use cartgo::math::{Real, vec3::Vec3};
use cartgo::output::ppm::save_p3;
use cartgo::render::camera::Camera;
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::render::{ray_color, render_pass};
use cartgo::render::render_settings::RenderSettings;
use cartgo::render::scene::Scene;

use crate::app::orbit::Orbit;

const ORBIT_SPEED: Real = 0.005;
const ZOOM_STEP: Real = 0.9;

struct Shared {
    camera: Mutex<Camera>,
    generation: AtomicU64,
    display: Mutex<Vec<u32>>,
    quit: AtomicBool,
}

pub fn run(spec: &ImageSpec, settings: &RenderSettings, scene: &Scene, mut camera: Camera, orbit: Orbit) {
    orbit.apply(&mut camera);
    let shared = Shared {
        camera: Mutex::new(camera),
        generation: AtomicU64::new(0),
        display: Mutex::new(vec![0; spec.pixel_count()]),
        quit: AtomicBool::new(false),
    };

    thread::scope(|scope| {
        scope.spawn(|| render_loop(spec, settings, scene, &shared));
        window_loop(spec, orbit, &shared);
        shared.quit.store(true, Ordering::Relaxed);
    });
}

fn render_loop(spec: &ImageSpec, settings: &RenderSettings, scene: &Scene, shared: &Shared) {
    let progress = ProgressBar::new(settings.samples_per_pixel as u64).with_style(
        ProgressStyle::with_template("[{elapsed_precise}] {wide_bar} {pos}/{len} passes ({per_sec}, ETA {eta})")
            .unwrap(),
    );
    let mut sums = vec![Color::BLACK; spec.pixel_count()];
    let mut pixels = vec![0u32; spec.pixel_count()];
    let mut passes = 0;
    let mut generation = u64::MAX;

    while !shared.quit.load(Ordering::Relaxed) {
        let (camera, current) = {
            let camera = shared.camera.lock().unwrap();
            (*camera, shared.generation.load(Ordering::Relaxed))
        };
        if current != generation {
            generation = current;
            passes = 0;
            sums.fill(Color::BLACK);
            progress.reset();
        }
        if passes == settings.samples_per_pixel {
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        let shade = |s, t, rng: &mut _| ray_color(&camera.get_ray(s, t, rng), scene, settings, rng);
        let cancel = || shared.quit.load(Ordering::Relaxed);
        if !render_pass(spec, settings.seed, passes, &mut sums, &shade, &cancel) {
            continue;
        }
        passes += 1;
        progress.inc(1);

        for (pixel, &sum) in pixels.iter_mut().zip(&sums) {
            let [r, g, b] = (sum / passes as Real).to_rgb8();
            *pixel = (r as u32) << 16 | (g as u32) << 8 | b as u32;
        }
        std::mem::swap(&mut *shared.display.lock().unwrap(), &mut pixels);

        if passes == settings.samples_per_pixel {
            progress.finish();
            let image: Vec<Color> = sums.iter().map(|&sum| sum / passes as Real).collect();
            if let Err(e) = save_p3("image.ppm", spec.width(), spec.height(), &image) {
                eprintln!("could not save image.ppm: {e}");
            }
        }
    }
}

// Right drag orbits, middle drag pans, WASD moves the target along the ground, Q/E down/up, scroll zooms
fn window_loop(spec: &ImageSpec, mut orbit: Orbit, shared: &Shared) {
    let vfov = shared.camera.lock().unwrap().vfov();
    let (width, height) = (spec.width() as usize, spec.height() as usize);
    let mut window = Window::new("caRTgo", width, height, WindowOptions::default())
        .unwrap_or_else(|e| panic!("could not open window: {e}"));
    window.set_target_fps(60);

    let mut buffer = vec![0u32; spec.pixel_count()];
    let mut last_mouse = window.get_mouse_pos(MouseMode::Pass);
    let mut last_frame = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = last_frame.elapsed().as_secs_f32();
        last_frame = Instant::now();
        let mut moved = false;

        let mouse = window.get_mouse_pos(MouseMode::Pass);
        if let (Some((x, y)), Some((last_x, last_y))) = (mouse, last_mouse) {
            let (dx, dy) = (x - last_x, y - last_y);
            if dx != 0.0 || dy != 0.0 {
                if window.get_mouse_down(MouseButton::Right) {
                    orbit.rotate(-dx * ORBIT_SPEED, dy * ORBIT_SPEED);
                    moved = true;
                }
                else if window.get_mouse_down(MouseButton::Middle) {
                    // World units per pixel at the target's distance, so the point under the cursor follows it
                    let scale = 2.0 * (vfov.to_radians() / 2.0).tan() * orbit.distance / height as Real;
                    orbit.pan(dx * scale, dy * scale);
                    moved = true;
                }
            }
        }
        last_mouse = mouse;

        let mut direction = Vec3::default();
        for (key, dir) in [
            (Key::W, orbit.forward()),
            (Key::S, -orbit.forward()),
            (Key::D, orbit.right()),
            (Key::A, -orbit.right()),
            (Key::E, Vec3::Z),
            (Key::Q, -Vec3::Z),
        ] {
            if window.is_key_down(key) {
                direction += dir;
            }
        }
        if direction != Vec3::default() {
            orbit.target += direction * (orbit.distance * dt);
            moved = true;
        }

        if let Some((_, scroll)) = window.get_scroll_wheel() {
            if scroll != 0.0 {
                orbit.distance *= if scroll > 0.0 {
                    ZOOM_STEP
                }
                else {
                    1.0 / ZOOM_STEP
                };
                moved = true;
            }
        }

        if moved {
            let mut camera = shared.camera.lock().unwrap();
            orbit.apply(&mut camera);
            shared.generation.fetch_add(1, Ordering::Relaxed);
        }

        buffer.copy_from_slice(&shared.display.lock().unwrap());
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}
