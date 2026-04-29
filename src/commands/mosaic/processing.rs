use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};

use super::MosaicMode;
use crate::Result;
use crate::commands::palette::{DitherMethod, Palette};

/// Calculate the dimensions of the image after resizing according to max_long_side.
fn calculate_dimensions(width: u32, height: u32, max_long_side: u32) -> (u32, u32) {
    let new_width;
    let new_height;

    if width > height {
        new_width = max_long_side;
        new_height = (height as f32 * (max_long_side as f32 / width as f32)).round() as u32;
    } else {
        new_height = max_long_side;
        new_width = (width as f32 * (max_long_side as f32 / height as f32)).round() as u32;
    }
    (new_width, new_height)
}

/// Resizes an image so that its longest side matches `max_long_side`,
/// maintaining the aspect ratio.
pub fn resize_to_max_long_side(
    img: DynamicImage,
    max_long_side: u32,
    mode: &MosaicMode,
) -> Result<DynamicImage> {
    let (width, height) = img.dimensions();
    let (new_width, new_height) = calculate_dimensions(width, height, max_long_side);
    let filter_type = match mode {
        MosaicMode::Sharp => FilterType::Nearest,
        MosaicMode::Smooth => FilterType::Lanczos3,
    };
    Ok(img.resize_exact(new_width, new_height, filter_type))
}

/// Quantizes the colors of an image to the specified number of colors.
/// Reuses the unified palette logic from the palette module.
pub fn quantize_colors(
    img: DynamicImage,
    colors: u8,
    mode: MosaicMode,
    dither: Option<DitherMethod>,
) -> Result<DynamicImage> {
    // 1. Generate palette (sampled for speed)
    let palette = Palette::extract(&img, colors, true)?;

    // 2. Apply palette to the full image
    // Use the provided dither method if it exists, otherwise fall back to mode-based default
    let dither_method = dither.unwrap_or(match mode {
        MosaicMode::Smooth => DitherMethod::FloydSteinberg,
        MosaicMode::Sharp => DitherMethod::None,
    });

    palette.apply(img, dither_method)
}

#[cfg(test)]
mod tests {
    use image::{ImageBuffer, Rgba};

    use super::*;

    #[test]
    fn test_calculate_dimensions_landscape() {
        let (w, h) = calculate_dimensions(1920, 1080, 64);
        assert_eq!((w, h), (64, 36));
    }

    #[test]
    fn test_calculate_dimensions_portrait() {
        let (w, h) = calculate_dimensions(1080, 1920, 64);
        assert_eq!((w, h), (36, 64));
    }

    #[test]
    fn test_calculate_dimensions_square() {
        let (w, h) = calculate_dimensions(100, 100, 64);
        assert_eq!((w, h), (64, 64));
    }

    #[test]
    fn test_resize_to_max_long_side_landscape() {
        let img = DynamicImage::ImageRgba8(ImageBuffer::<Rgba<u8>, _>::new(200, 100));
        let resized = resize_to_max_long_side(img, 50, &MosaicMode::Sharp).unwrap();
        assert_eq!(resized.dimensions(), (50, 25));
    }

    #[test]
    fn test_resize_to_max_long_side_portrait() {
        let img = DynamicImage::ImageRgba8(ImageBuffer::<Rgba<u8>, _>::new(100, 200));
        let resized = resize_to_max_long_side(img, 50, &MosaicMode::Sharp).unwrap();
        assert_eq!(resized.dimensions(), (25, 50));
    }

    #[test]
    fn test_quantize_colors_sharp_mode() {
        let mut img_buf = ImageBuffer::new(2, 2);
        img_buf.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        img_buf.put_pixel(1, 0, Rgba([0, 255, 0, 255]));
        img_buf.put_pixel(0, 1, Rgba([0, 0, 255, 255]));
        img_buf.put_pixel(1, 1, Rgba([255, 255, 255, 255]));
        let img = DynamicImage::ImageRgba8(img_buf);

        let result = quantize_colors(img, 2, MosaicMode::Sharp, None).unwrap();
        assert_eq!(result.dimensions(), (2, 2));

        let rgba = result.into_rgba8();
        let unique_colors: std::collections::HashSet<_> = rgba.pixels().map(|p| p.0).collect();
        assert!(unique_colors.len() <= 2);
    }

    #[test]
    fn test_quantize_colors_smooth_mode() {
        let mut img_buf = ImageBuffer::new(4, 4);
        for (x, y, pixel) in img_buf.enumerate_pixels_mut() {
            *pixel = Rgba([(x * 60) as u8, (y * 60) as u8, 128, 255]);
        }
        let img = DynamicImage::ImageRgba8(img_buf);

        // check if the dimensions are not changed
        let result = quantize_colors(img, 4, MosaicMode::Smooth, None).unwrap();
        assert_eq!(result.dimensions(), (4, 4));

        let rgba = result.into_rgba8();
        let unique_colors: std::collections::HashSet<_> = rgba.pixels().map(|p| p.0).collect();
        assert!(unique_colors.len() <= 4);
    }
}
