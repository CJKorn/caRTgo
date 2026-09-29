use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut window = Window::new("caRTgo", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| panic!("could not open window: {e}"));
    window.set_target_fps(60);

    // 0x00RRGGBB
    let buffer = vec![0x0020_2020u32; WIDTH * HEIGHT];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
