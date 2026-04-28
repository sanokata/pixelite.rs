# pixelite

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![AI Assisted](https://img.shields.io/badge/AI-Assisted-blue.svg)](#)

`pixelite` is a Rust-based command-line tool for converting images into pixel art.
It provides optimized image processing pipelines for various use cases, such as clean edge processing for characters and smooth color reduction for backgrounds.

[日本語版はこちら (Japanese version available here)](README_ja.md)

## Features

- **Two Processing Modes**:
  - `Sharp`: Designed to keep outlines sharp and handle transparency correctly. Applies post-processing such as orphan pixel removal, L-shape correction (pixel perfection), and outlining.
  - `Smooth`: Designed to maintain smooth gradients. Performs natural color reduction using dithering and Gaussian blur.
- **Smart Crop**: Automatically crops the center of the image into a square.
- **Advanced Image Processing**: Optimal palette selection using the KMeans algorithm and luminance calculation based on Rec. 709 weights.

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

### Basic Command

```bash
# Convert an image to 64x64 pixel art
pixelite mosaic --input photo.jpg --output out.png --size 64
```

### Crop and Process in Character Mode

```bash
# Crop the center to a square and apply character mode at 32x32 resolution
pixelite mosaic -i input.png -o output.png -s 32 -m sharp --crop
```

### Options

Run `pixelite mosaic --help` for more details.

| Option | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--input` | `-i` | (Required) | Path to the input image |
| `--output` | `-o` | (Required) | Path to the output image (PNG recommended) |
| `--size` | `-s` | `64` | Resolution of the longest side (pixels) |
| `--colors` | `-c` | `16` | Maximum number of colors to use |
| `--mode` | `-m` | `character` | Processing mode (`character` or `background`) |
| `--crop` | | `false` | Whether to crop the center of the image into a square |
| `--scale` | `-S` | `1` | Output scale factor (multiplies physical pixel size) |
| `--verbose`| `-v` | `false` | Show detailed logs (global option) |

## Development

### Running Tests

```bash
cargo test
```

## License

This project is licensed under the [MIT License](LICENSE).

---

**Note**: Generative AI was used to assist in the development of this project. However, all code has been reviewed, tested, and the author takes full responsibility for the final quality and functionality.
