# pixelite

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![AI Assisted](https://img.shields.io/badge/AI-Assisted-blue.svg)](#)

`pixelite` is a Rust-based command-line tool for converting images into pixel art.
It provides optimized image processing pipelines for various use cases, such as clean edge processing for character sprites and smooth color reduction for backgrounds.

[日本語版はこちら (Japanese version available here)](README_ja.md)

## Features

- **Two Processing Modes** (`mosaic` command):
  - `sharp`: Designed to keep outlines sharp and handle transparency correctly. Applies post-processing such as orphan pixel removal, L-shape correction (pixel perfection), and outlining.
  - `smooth`: Designed to maintain smooth gradients. Performs natural color reduction using Floyd-Steinberg dithering and Gaussian blur.
- **Dithering Control**: Override the mode-default dithering method with `--dither` (`none`, `floyd-steinberg`, `ordered`).
- **Palette Management** (`palette` command): Extract optimal color palettes from images or apply existing palettes to new images.
- **Smart Crop**: Automatically crops the center of the image into a square before processing.
- **Advanced Color Quantization**: Optimal palette selection using the KMeans algorithm and luminance calculation based on Rec. 709 weights.

## Installation

If you have Rust installed, you can build and install it using `cargo`.

```bash
# Clone the repository
git clone https://github.com/sanokata/pixelite.rs.git
cd pixelite.rs

# Build and install
cargo install --path .
```

## Usage

### `mosaic` — Convert an Image to Pixel Art

```bash
# Convert an image to 64x64 pixel art (sharp mode, default)
pixelite mosaic --input photo.jpg --output out.png --size 64

# Crop the center to a square and apply sharp mode at 32x32 resolution
pixelite mosaic -i input.png -o output.png -s 32 -m sharp --crop

# Smooth mode for backgrounds, scaled up 4x for display
pixelite mosaic -i background.jpg -o bg_pixel.png -s 128 -m smooth -S 4

# Sharp mode with ordered dithering (overrides mode default)
pixelite mosaic -i sprite.png -o sprite_pixel.png -m sharp --dither ordered
```

#### Options

Run `pixelite mosaic --help` for full details.

| Option | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--input` | `-i` | (Required) | Path to the input image |
| `--output` | `-o` | (Required) | Path to the output image (PNG recommended) |
| `--size` | `-s` | `64` | Resolution of the longest side (pixels) |
| `--colors` | `-c` | `16` | Maximum number of colors to use |
| `--mode` | `-m` | `sharp` | Processing mode (`sharp` or `smooth`) |
| `--dither` | `-d` | (mode default) | Dithering method override (`none`, `floyd-steinberg`, `ordered`) |
| `--crop` | | `false` | Crop the center of the image into a square before processing |
| `--scale` | `-S` | `1` | Output scale factor (multiplies physical pixel size per dot) |
| `--verbose` | `-v` | `false` | Show detailed logs (global option) |

---

### `palette` — Extract or Apply Color Palettes

#### Extract a palette from an image

```bash
# Extract 16 colors and save as a PNG swatch
pixelite palette extract --input photo.jpg --output palette.png --colors 16

# Extract and save as a GIMP palette file
pixelite palette extract -i photo.jpg -o palette.gpl -c 8
```

| Option | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--input` | `-i` | (Required) | Path to the input image |
| `--output` | `-o` | (Required) | Output palette path (`.png` or `.gpl`) |
| `--colors` | `-c` | `16` | Number of colors to extract |

#### Apply a palette to an image

```bash
# Apply a palette with no dithering
pixelite palette apply --input photo.jpg --palette palette.png --output result.png

# Apply with Floyd-Steinberg dithering
pixelite palette apply -i photo.jpg -p palette.gpl -o result.png --method floyd-steinberg

# Apply with ordered (Bayer 4x4) dithering
pixelite palette apply -i photo.jpg -p palette.png -o result.png --method ordered
```

| Option | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--input` | `-i` | (Required) | Path to the input image |
| `--palette` | `-p` | (Required) | Palette file path (`.png` or `.gpl`) |
| `--output` | `-o` | (Required) | Path to the output image |
| `--method` | `-m` | `none` | Dithering method (`none`, `floyd-steinberg`, `ordered`) |

## Development

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the [MIT License](LICENSE).

---

**Note**: Generative AI was used to assist in the development of this project. However, all code has been reviewed, tested, and the author takes full responsibility for the final quality and functionality.
