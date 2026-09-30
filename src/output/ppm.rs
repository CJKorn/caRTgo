use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

use crate::color::Color;

pub fn write_p3<W: Write>(out: &mut W, width: u32, height: u32, pixels: &[Color]) -> io::Result<()> {
    assert_eq!(
        pixels.len(),
        width as usize * height as usize,
        "pixel buffer does not match {width}x{height}"
    );

    writeln!(out, "P3\n{width} {height}\n255")?;
    for row in pixels.chunks(width as usize) {
        for pixel in row {
            let [r, g, b] = pixel.to_rgb8();
            writeln!(out, "{r} {g} {b}")?;
        }
    }
    out.flush()
}

pub fn save_p3(path: impl AsRef<Path>, width: u32, height: u32, pixels: &[Color]) -> io::Result<()> {
    let mut out = BufWriter::new(File::create(path)?);
    write_p3(&mut out, width, height, pixels)
}

pub fn write_p6<W: Write>(out: &mut W, width: u32, height: u32, pixels: &[Color]) -> io::Result<()> {
    assert_eq!(
        pixels.len(),
        width as usize * height as usize,
        "pixel buffer does not match {width}x{height}"
    );

    write!(out, "P6\n{width} {height}\n255\n")?;
    let bytes: Vec<u8> = pixels.iter().flat_map(|p| p.to_rgb8()).collect();
    out.write_all(&bytes)?;
    out.flush()
}

pub fn save_p6(path: impl AsRef<Path>, width: u32, height: u32, pixels: &[Color]) -> io::Result<()> {
    let mut out = BufWriter::new(File::create(path)?);
    write_p6(&mut out, width, height, pixels)
}
