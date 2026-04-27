use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};

use crate::Result;

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
    return (new_width, new_height);
}

/// Resizes an image so that its longest side matches `max_long_side`,
/// maintaining the aspect ratio.
pub fn resize_to_max_long_side(img: DynamicImage, max_long_side: u32) -> Result<DynamicImage> {
    let (width, height) = img.dimensions();
    let (new_width, new_height) = calculate_dimensions(width, height, max_long_side);
    Ok(img.resize_exact(new_width, new_height, FilterType::Lanczos3))
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
        let resized = resize_to_max_long_side(img, 50).unwrap();
        assert_eq!(resized.dimensions(), (50, 25));
    }

    #[test]
    fn test_resize_to_max_long_side_portrait() {
        let img = DynamicImage::ImageRgba8(ImageBuffer::<Rgba<u8>, _>::new(100, 200));
        let resized = resize_to_max_long_side(img, 50).unwrap();
        assert_eq!(resized.dimensions(), (25, 50));
    }
}
