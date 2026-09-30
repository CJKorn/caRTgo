mod app;

use std::io;
use std::process;

use cartgo::color::Color;
use cartgo::loader::gltf::load_gltf;
use cartgo::math::{EPS, Real, interval::Interval, ray::Ray, vec3::Vec3};
use cartgo::output::ppm::save_p3;
use cartgo::render::image_spec::ImageSpec;
use cartgo::render::light::Sky;
use cartgo::render::render::{ray_color, render};
use cartgo::render::render_settings::RenderSettings;
use cartgo::render::scene::Scene;

use crate::app::cli::{self, Command, Mode, USAGE};
use crate::app::menu::{self, Settings};
use crate::app::orbit::Orbit;
use crate::app::progress_bar;

const SCENE_DIR: &str = "assets/scenes";

fn main() {
    let args = match cli::parse(std::env::args().skip(1)) {
        Ok(Command::Run(args)) => args,
        Ok(Command::Help) => {
            println!("{USAGE}");
            return;
        }
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            process::exit(2);
        }
    };
    if let Err(e) = run(args) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run(args: cli::Args) -> io::Result<()> {
    let scene_path = match &args.scene {
        Some(path) => path.clone(),
        None => {
            let files = menu::scene_files(SCENE_DIR)?;
            if files.is_empty() {
                return Err(io::Error::new(io::ErrorKind::NotFound, format!("no scenes in {SCENE_DIR}")));
            }
            let names: Vec<String> = files.iter().map(|f| f.file_stem().unwrap().to_string_lossy().into()).collect();
            files[menu::choose("Scene", &names)?].clone()
        }
    };

    let mode = match args.mode {
        Some(mode) => mode,
        None => {
            let modes = ["Open the viewport".to_string(), "Render an image".to_string()];
            [Mode::Viewport, Mode::Image][menu::choose("Mode", &modes)?]
        }
    };

    let mut scene = Scene::new();
    load_gltf(&scene_path, &mut scene)
        .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", scene_path.display())))?;
    scene.set_sky(Sky {
        horizon: Color::new(0.35, 0.4, 0.45),
        zenith: Color::new(0.15, 0.25, 0.5),
    });

    let camera_index = match args.camera {
        Some(index) => index,
        None if scene.cameras().is_empty() => 0,
        None => {
            let names: Vec<String> = scene.cameras().iter().map(|c| c.name.clone()).collect();
            menu::choose("Camera", &names)?
        }
    };
    let scene_camera = scene.cameras().get(camera_index).cloned().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} has no camera {camera_index}", scene_path.display()),
        )
    })?;

    let (mut options, all_given) = Settings::from_args(&args);
    if !all_given {
        menu::edit_settings(&mut options)?;
    }

    let spec = ImageSpec::new(options.width, options.height);
    let camera = scene_camera.to_camera(&spec);
    let settings = RenderSettings {
        samples_per_pixel: options.samples,
        max_depth: options.depth,
        seed: options.seed,
        ..Default::default()
    };

    match mode {
        Mode::Viewport => {
            // Orbit around whatever the camera is looking at
            let forward = scene_camera.rotation.rotate(-Vec3::Z);
            let view_ray = Ray::new(scene_camera.position, forward, 0.0);
            let distance = scene
                .hit(&view_ray, Interval::new(EPS, Real::INFINITY))
                .map_or(10.0, |hit| hit.t);
            let orbit = Orbit::from_position(scene_camera.position, scene_camera.position + distance * forward);
            app::viewport::run(&spec, &settings, &scene, camera, orbit, &options.output);
        }
        Mode::Image => {
            let progress = progress_bar(settings.samples_per_pixel);
            let pixels = render(
                &spec,
                &settings,
                |s, t, rng| ray_color(&camera.get_ray(s, t, rng), &scene, &settings, rng),
                |_| progress.inc(1),
            );
            progress.finish();
            save_p3(&options.output, spec.width(), spec.height(), &pixels)?;
            println!("Saved {}", options.output.display());
        }
    }
    Ok(())
}
