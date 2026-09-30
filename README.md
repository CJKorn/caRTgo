# Video Link
https://youtu.be/nA_OFiqOzOg
*(Sorry it's 8 minutes long :P )*

# Document
[Programming Assignment 1.pdf](<Programming Assignment 1.pdf>)

# Building
Of course, make sure you have Rust installed via [rustup](https://rustup.rs) or whatever package manager you're most comfortable with.
```bash
cargo build --release
```

## Running

Run from the project root, since scenes are looked up in `assets/scenes/`.

### Interactive

```bash
cargo run --release
```

With no arguments the program asks, using arrow keys and Enter:

1. **Scene**: any `.glb` / `.gltf` file in `assets/scenes/`
2. **Mode**: open the live viewport, or render an image to a file
3. **Camera**: only asked if the scene has more than one
4. **Render settings**: width, height, samples, max bounces, seed and output file. Select a line to change it, then **Start**.

### Command line

Anything given as an argument skips that question. For example:

```bash
# Render to a file without any prompts
cargo run --release -- assets/scenes/CornellBox.glb --render --camera 0 \
    --width 1280 --height 720 --samples 256 --depth 12 --seed 42 -o cornell.png

# Open the viewport, asking only for the render settings
cargo run --release -- assets/scenes/CornellBoxless.glb --window
```

| Option | Description | Default |
|---|---|---|
| `SCENE.glb` | scene file to load | asked |
| `--window` | open the live viewport | asked |
| `--render` | render an image to a file | asked |
| `--camera N` | camera index in the scene, from 0 | asked if more than one |
| `--width N` | image width | 1920 |
| `--height N` | image height | 1080 |
| `--samples N` | samples per pixel | 1000 |
| `--depth N` | maximum bounces | 20 |
| `--seed N` | random seed | 42 |
| `-o`, `--output FILE` | output image, `.png` or `.ppm` | `image.png` |
| `-h`, `--help` | show help | |

The render settings menu is skipped only when all of `--width`, `--height`, `--samples`, `--depth`, `--seed` and `--output` are given.

### Viewport controls

| Input | Action |
|---|---|
| Right mouse drag | orbit around the target |
| Middle mouse drag | pan |
| W / A / S / D | move the target along the ground |
| Q / E | move the target down / up |
| Scroll wheel | zoom |
| Esc | quit |

The image keeps refining while the camera is still, and restarts when it moves. Once all samples are done it is saved to the output file.

## Scenes

Scenes are glTF 2.0 files exported from Blender. When exporting, enable **Punctual Lights** and **Cameras** so they are included.
