use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};

use cartgo::output;

use crate::app::cli::Args;

pub struct Settings {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub depth: u32,
    pub seed: u64,
    pub output: PathBuf,
}

impl Settings {
    // Also returns whether every setting came from the command line
    pub fn from_args(args: &Args) -> (Self, bool) {
        let all_given = args.width.is_some()
            && args.height.is_some()
            && args.samples.is_some()
            && args.depth.is_some()
            && args.seed.is_some()
            && args.output.is_some();
        let settings = Self {
            width: args.width.unwrap_or(1920),
            height: args.height.unwrap_or(1080),
            samples: args.samples.unwrap_or(1000),
            depth: args.depth.unwrap_or(20),
            seed: args.seed.unwrap_or(42),
            output: args.output.clone().unwrap_or_else(|| PathBuf::from("image.png")),
        };
        (settings, all_given)
    }
}

pub fn scene_files(dir: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "glb" || ext == "gltf"))
        .collect();
    files.sort();
    Ok(files)
}

// Skips the question when there's only one option
pub fn choose(prompt: &str, items: &[String]) -> io::Result<usize> {
    if items.len() == 1 {
        return Ok(0);
    }
    Ok(Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()?)
}

pub fn edit_settings(settings: &mut Settings) -> io::Result<()> {
    let theme = ColorfulTheme::default();
    let mut selected = 0;
    loop {
        let items = [
            format!("Width          {}", settings.width),
            format!("Height         {}", settings.height),
            format!("Samples        {}", settings.samples),
            format!("Max bounces    {}", settings.depth),
            format!("Seed           {}", settings.seed),
            format!("Output         {}", settings.output.display()),
            "Start".to_string(),
        ];
        selected = Select::with_theme(&theme)
            .with_prompt("Render settings")
            .items(&items)
            .default(selected)
            .interact()?;

        match selected {
            0 => settings.width = ask_positive(&theme, "Width", settings.width)?,
            1 => settings.height = ask_positive(&theme, "Height", settings.height)?,
            2 => settings.samples = ask_positive(&theme, "Samples per pixel", settings.samples)?,
            3 => settings.depth = ask(&theme, "Max bounces", settings.depth)?,
            4 => settings.seed = ask(&theme, "Seed", settings.seed)?,
            5 => {
                let output: String = Input::with_theme(&theme)
                    .with_prompt("Output file")
                    .default(settings.output.display().to_string())
                    .validate_with(|path: &String| {
                        if output::is_supported(path) { Ok(()) } else { Err("use a .png or .ppm file") }
                    })
                    .interact_text()?;
                settings.output = PathBuf::from(output);
            }
            _ => return Ok(()),
        }
    }
}

fn ask<T>(theme: &ColorfulTheme, prompt: &str, current: T) -> io::Result<T>
where
    T: Clone + ToString + std::str::FromStr,
    T::Err: ToString,
{
    Ok(Input::with_theme(theme).with_prompt(prompt).default(current).interact_text()?)
}

fn ask_positive(theme: &ColorfulTheme, prompt: &str, current: u32) -> io::Result<u32> {
    Ok(Input::with_theme(theme)
        .with_prompt(prompt)
        .default(current)
        .validate_with(|n: &u32| if *n > 0 { Ok(()) } else { Err("must be at least 1") })
        .interact_text()?)
}
