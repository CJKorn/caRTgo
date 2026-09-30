pub mod png;
pub mod ppm;

use std::io;
use std::path::Path;

use crate::color::Color;

pub fn is_supported(path: impl AsRef<Path>) -> bool {
    matches!(extension(path.as_ref()).as_deref(), Some("png" | "ppm"))
}

// Format from the extension: .png, or .ppm (binary P6)
pub fn save(path: impl AsRef<Path>, width: u32, height: u32, pixels: &[Color]) -> io::Result<()> {
    let path = path.as_ref();
    match extension(path).as_deref() {
        Some("png") => png::save_png(path, width, height, pixels),
        Some("ppm") => ppm::save_p6(path, width, height, pixels),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{}: unsupported format, use .png or .ppm", path.display()),
        )),
    }
}

fn extension(path: &Path) -> Option<String> {
    path.extension().map(|e| e.to_string_lossy().to_lowercase())
}
