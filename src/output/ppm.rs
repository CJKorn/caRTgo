//TODO: Change to P6 later

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
