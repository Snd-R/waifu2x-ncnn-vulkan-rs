## Building

required dependencies:

- cmake
- g++
- vulkan loader library
- ncnn
- glslang

1. run `git submodule update --init --recursive` to download subprojects required for build
2. set GLSLANG_TARGET_DIR environment variable (ubuntu: `/usr/lib/x86_64-linux-gnu/cmake/` arch linux: `/usr/lib/cmake`)
3. run `GLSLANG_TARGET_DIR=/usr/lib/cmake cargo build --release`

## Usage

```rust
use waifu2x_ncnn_vulkan_rs::Waifu2x;

fn main() {
    let image = image::open("image.png")?;

    let waifu2x = Waifu2x::new(
        config.gpuid,
        config.noise,
        config.scale,
        config.model,
        config.tile_size,
        config.tta_mode,
        config.num_threads,
        config.models_path,
    );

    waifu2x.proc_image(image).save("output.png");
}
```