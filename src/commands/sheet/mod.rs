use std::path::{Path, PathBuf};

use clap::Args;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba, imageops};

use crate::Result;

/// The arguments of sheet sub-command
#[derive(Args, Debug)]
#[command(about = "Assembles multiple images into a spritesheet.")]
pub struct SheetArgs {
    /// Input image path(s) or glob pattern(s) (e.g. "sprites/*.png")
    #[arg(short, long, required = true)]
    pub input: Vec<String>,

    /// Output spritesheet image file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Number of columns in the spritesheet grid (default: ceil(sqrt(N)))
    #[arg(short, long)]
    pub columns: Option<u32>,

    /// Pad images smaller than the largest to the same cell size using transparency
    #[arg(long, default_value_t = false)]
    pub padding: bool,
}

pub fn run(args: SheetArgs) -> Result<()> {
    if args.columns == Some(0) {
        return Err("--columns must be at least 1".into());
    }
    let inputs = super::expand_inputs(&args.input)?;
    if inputs.is_empty() {
        return Err("no input files found".into());
    }
    let images = load_images(inputs)?;
    let (cell_w, cell_h) = resolve_cell_size(&images, args.padding)?;
    let cols = grid_columns(images.len() as u32, args.columns);
    let canvas = build_canvas(images, cell_w, cell_h, cols);
    save_image(DynamicImage::ImageRgba8(canvas), &args.output)
}

fn load_images(inputs: Vec<PathBuf>) -> Result<Vec<(PathBuf, DynamicImage)>> {
    inputs
        .into_iter()
        .map(|path| {
            let img = image::open(&path)?;
            Ok((path, img))
        })
        .collect()
}

fn resolve_cell_size(images: &[(PathBuf, DynamicImage)], padding: bool) -> Result<(u32, u32)> {
    let sizes: Vec<(PathBuf, u32, u32)> = images
        .iter()
        .map(|(p, img)| (p.clone(), img.width(), img.height()))
        .collect();
    if !padding
        && let Some(err) = check_uniform_size(&sizes) {
            return Err(err.into());
        }
    let w = sizes.iter().map(|(_, w, _)| *w).max().unwrap();
    let h = sizes.iter().map(|(_, _, h)| *h).max().unwrap();
    Ok((w, h))
}

fn build_canvas(
    images: Vec<(PathBuf, DynamicImage)>,
    cell_w: u32,
    cell_h: u32,
    cols: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let n = images.len() as u32;
    let rows = n.div_ceil(cols);
    let mut canvas = ImageBuffer::<Rgba<u8>, _>::new(cols * cell_w, rows * cell_h);
    for (i, (_, img)) in images.into_iter().enumerate() {
        let x = ((i as u32) % cols) * cell_w;
        let y = ((i as u32) / cols) * cell_h;
        imageops::overlay(&mut canvas, &img.into_rgba8(), x as i64, y as i64);
    }
    canvas
}

fn save_image(img: DynamicImage, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    let format = ImageFormat::from_path(output).unwrap_or(ImageFormat::Png);
    img.save_with_format(output, format)?;
    Ok(())
}

fn grid_columns(n: u32, columns_override: Option<u32>) -> u32 {
    columns_override
        .unwrap_or_else(|| (n as f64).sqrt().ceil() as u32)
        .max(1)
}

fn check_uniform_size(images: &[(PathBuf, u32, u32)]) -> Option<String> {
    let unique: std::collections::HashSet<(u32, u32)> =
        images.iter().map(|(_, w, h)| (*w, *h)).collect();
    if unique.len() <= 1 {
        return None;
    }
    let mut msg = "size mismatch: all images must be the same size\n".to_string();
    for (path, w, h) in images {
        msg.push_str(&format!("  {}: {}x{}\n", path.display(), w, h));
    }
    Some(msg.trim_end().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn solid(w: u32, h: u32, color: [u8; 4]) -> DynamicImage {
        let mut buf = ImageBuffer::new(w, h);
        for pixel in buf.pixels_mut() {
            *pixel = Rgba(color);
        }
        DynamicImage::ImageRgba8(buf)
    }

    #[test]
    fn test_grid_columns_auto_square() {
        assert_eq!(grid_columns(4, None), 2);
    }

    #[test]
    fn test_grid_columns_auto_non_square() {
        // ceil(sqrt(5)) = 3
        assert_eq!(grid_columns(5, None), 3);
    }

    #[test]
    fn test_grid_columns_auto_single() {
        assert_eq!(grid_columns(1, None), 1);
    }

    #[test]
    fn test_grid_columns_override() {
        assert_eq!(grid_columns(9, Some(4)), 4);
    }

    #[test]
    fn test_check_uniform_size_ok() {
        let images = vec![
            (PathBuf::from("a.png"), 32, 32),
            (PathBuf::from("b.png"), 32, 32),
        ];
        assert!(check_uniform_size(&images).is_none());
    }

    #[test]
    fn test_check_uniform_size_single() {
        assert!(check_uniform_size(&[(PathBuf::from("a.png"), 32, 32)]).is_none());
    }

    #[test]
    fn test_check_uniform_size_mismatch_reports_all_files() {
        let images = vec![
            (PathBuf::from("a.png"), 32, 32),
            (PathBuf::from("b.png"), 32, 48),
            (PathBuf::from("c.png"), 64, 64),
        ];
        let err = check_uniform_size(&images).unwrap();
        assert!(err.contains("a.png") && err.contains("32x32"));
        assert!(err.contains("b.png") && err.contains("32x48"));
        assert!(err.contains("c.png") && err.contains("64x64"));
    }

    #[test]
    fn test_build_canvas_dimensions() {
        // 4 images of 8x8 → 2x2 grid → 16x16 canvas
        let images = (0..4)
            .map(|_| (PathBuf::new(), solid(8, 8, [255, 0, 0, 255])))
            .collect();
        let canvas = build_canvas(images, 8, 8, 2);
        assert_eq!(canvas.dimensions(), (16, 16));
    }

    #[test]
    fn test_build_canvas_trailing_cell_is_transparent() {
        // 3 images of 4x4 → 2 cols, 2 rows → last cell transparent
        let images = (0..3)
            .map(|_| (PathBuf::new(), solid(4, 4, [255, 0, 0, 255])))
            .collect();
        let canvas = build_canvas(images, 4, 4, 2);
        assert_eq!(canvas.get_pixel(7, 7).0, [0, 0, 0, 0]);
    }

    #[test]
    fn test_build_canvas_padding_places_small_image_at_top_left() {
        // Small (2x2) in a 4x4 cell — top-left filled, bottom-right transparent
        let images = vec![(PathBuf::new(), solid(2, 2, [255, 0, 0, 255]))];
        let canvas = build_canvas(images, 4, 4, 1);
        assert_eq!(canvas.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(canvas.get_pixel(3, 3).0, [0, 0, 0, 0]);
    }
}
