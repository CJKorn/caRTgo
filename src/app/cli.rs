use std::path::PathBuf;
use std::str::FromStr;

pub const USAGE: &str = "\
Usage: cartgo [SCENE.glb] [options]
Anything not given on the command line is asked for in the terminal.

Options:
  --window          open the live viewport
  --render          render an image to a file
  --camera N        camera index in the scene, from 0
  --width N         image width (default 640)
  --height N        image height (default 360)
  --samples N       samples per pixel (default 1000)
  --depth N         maximum bounces (default 20)
  --seed N          random seed (default 42)
  -o, --output FILE output image (default image.ppm)
  -h, --help        show this help

The render settings menu is skipped when --width, --height, --samples, --depth, --seed and --output are all given.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Viewport,
    Image,
}

pub enum Command {
    Help,
    Run(Args),
}

#[derive(Default)]
pub struct Args {
    pub scene: Option<PathBuf>,
    pub mode: Option<Mode>,
    pub camera: Option<usize>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub samples: Option<u32>,
    pub depth: Option<u32>,
    pub seed: Option<u64>,
    pub output: Option<PathBuf>,
}

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let mut parsed = Args::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "--window" => set_mode(&mut parsed, Mode::Viewport)?,
            "--render" => set_mode(&mut parsed, Mode::Image)?,
            "--camera" => parsed.camera = Some(number(&arg, args.next())?),
            "--width" => parsed.width = Some(positive(&arg, args.next())?),
            "--height" => parsed.height = Some(positive(&arg, args.next())?),
            "--samples" => parsed.samples = Some(positive(&arg, args.next())?),
            "--depth" => parsed.depth = Some(number(&arg, args.next())?),
            "--seed" => parsed.seed = Some(number(&arg, args.next())?),
            "-o" | "--output" => {
                parsed.output = Some(PathBuf::from(args.next().ok_or(format!("{arg} needs a value"))?))
            }
            flag if flag.starts_with('-') => return Err(format!("unknown option {flag}")),
            path if parsed.scene.is_none() => parsed.scene = Some(PathBuf::from(path)),
            extra => return Err(format!("unexpected argument {extra}")),
        }
    }
    Ok(Command::Run(parsed))
}

fn set_mode(args: &mut Args, mode: Mode) -> Result<(), String> {
    if args.mode.is_some_and(|m| m != mode) {
        return Err("--window and --render can't be used together".to_string());
    }
    args.mode = Some(mode);
    Ok(())
}

fn number<T: FromStr>(flag: &str, value: Option<String>) -> Result<T, String> {
    let value = value.ok_or(format!("{flag} needs a value"))?;
    value.parse().map_err(|_| format!("{flag} expects a number, got {value}"))
}

fn positive(flag: &str, value: Option<String>) -> Result<u32, String> {
    match number(flag, value)? {
        0 => Err(format!("{flag} must be at least 1")),
        n => Ok(n),
    }
}
