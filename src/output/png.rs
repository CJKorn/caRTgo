use std::io;
use std::path::Path;

use image::{ImageFormat, RgbImage};

use crate::color::Color;

pub fn save_png(path: impl AsRef<Path>, width: u32, height: u32, pixels: &[Color]) -> io::Result<()> {
    let bytes: Vec<u8> = pixels.iter().flat_map(|p| p.to_rgb8()).collect();
    let image = RgbImage::from_raw(width, height, bytes)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "pixel buffer does not match image size"))?;
    image.save_with_format(path, ImageFormat::Png).map_err(io::Error::other)
}
