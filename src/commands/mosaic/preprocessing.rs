use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use libblur::{
    BlurImage, BlurImageMut, ConvolutionMode, EdgeMode2D, FastBlurChannels, GaussianBlurParams,
    ThreadingPolicy, gaussian_blur,
};

use super::MosaicMode;
use crate::Result;

/// Gaussian blur sigma applied during Smooth mode noise reduction.
const SMOOTH_BLUR_SIGMA: f32 = 1.0;
/// Saturation scale factor for Smooth mode (values < 1.0 reduce saturation).
const SMOOTH_SATURATION_FACTOR: f32 = 0.85;

/// Applies mode-appropriate preprocessing to the image before resizing and quantization.
///
/// - `Sharp`: Binarizes the alpha channel (0 or 255) to preserve hard edges.
/// - `Smooth`: Applies a gentle Gaussian blur to reduce noise and smooth gradients,
///   then lowers saturation slightly for more natural palette reduction.
pub fn preprocess(img: DynamicImage, mode: &MosaicMode) -> Result<DynamicImage> {
    match mode {
        MosaicMode::Sharp => preprocess_sharp(img),
        MosaicMode::Smooth => preprocess_smooth(img),
    }
}

/// Crops the image to a centered square based on the shortest side, if `should_crop` is true.
pub fn crop_to_square(mut img: DynamicImage, should_crop: bool) -> DynamicImage {
    if !should_crop {
        return img;
    }
    let (width, height) = img.dimensions();
    let size = width.min(height);

    let x = (width - size) / 2;
    let y = (height - size) / 2;

    img.crop(x, y, size, size)
}

/// Sharp mode: keep edges intact; only binarize alpha so quantization
/// does not create semi-transparent fringe pixels.
fn preprocess_sharp(img: DynamicImage) -> Result<DynamicImage> {
    // infalliable but returning Result for API uniformity with preprocess_background
    Ok(binarize_alpha(img))
}

/// Smooth mode: smooth noise with a Gaussian blur, then reduce saturation
/// so that the KMeans palette covers gradient areas more evenly.
fn preprocess_smooth(img: DynamicImage) -> Result<DynamicImage> {
    let img = apply_gaussian_blur(img, SMOOTH_BLUR_SIGMA)?;
    let img = apply_saturation(img, SMOOTH_SATURATION_FACTOR)?;
    Ok(img)
}

/// Binarizes the alpha channel: pixels with alpha >= 128 become fully opaque (255),
/// pixels with alpha < 128 become fully transparent (0).
fn binarize_alpha(img: DynamicImage) -> DynamicImage {
    let mut rgba = img.into_rgba8();
    for pixel in rgba.pixels_mut() {
        pixel[3] = if pixel[3] >= 128 { 255 } else { 0 };
    }
    DynamicImage::ImageRgba8(rgba)
}

/// Applies a Gaussian blur with the given `sigma` to an image.
fn apply_gaussian_blur(img: DynamicImage, sigma: f32) -> Result<DynamicImage> {
    let rgba = img.into_rgba8();
    let blurred = gaussian_blur_rgba(&rgba, sigma)?;
    Ok(DynamicImage::ImageRgba8(blurred))
}

/// Executes the libblur Gaussian blur pipeline on an RGBA8 buffer.
fn gaussian_blur_rgba(src: &image::RgbaImage, sigma: f32) -> Result<image::RgbaImage> {
    let (width, height) = src.dimensions();
    let src_view = BlurImage::borrow(src.as_raw(), width, height, FastBlurChannels::Channels4);
    let mut dst_buf = vec![0u8; src.len()];
    let mut dst = BlurImageMut::borrow(&mut dst_buf, width, height, FastBlurChannels::Channels4);
    gaussian_blur(
        &src_view,
        &mut dst,
        GaussianBlurParams::new_from_sigma(sigma as f64),
        EdgeMode2D::default(),
        ThreadingPolicy::Single,
        ConvolutionMode::default(),
    )?;
    image::ImageBuffer::from_raw(width, height, dst_buf)
        .ok_or_else(|| "Failed to construct blurred image buffer".into())
}

/// Scales the saturation of every pixel by `factor` (0.0 = grayscale, 1.0 = no change).
/// Operates in RGB space via a simple desaturation blend with luminance.
fn apply_saturation(img: DynamicImage, factor: f32) -> Result<DynamicImage> {
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8();

    let adjusted = ImageBuffer::from_fn(width, height, |x, y| {
        let p = rgba.get_pixel(x, y);
        let [r, g, b, a] = p.0;

        // Calculate perceived brightness (luminance) using Rec. 709 weights.
        // This ensures natural-looking desaturation based on human eye sensitivity.
        let luma = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
        let blend = |channel: u8| -> u8 {
            (luma + factor * (channel as f32 - luma))
                .round()
                .clamp(0.0, 255.0) as u8
        };

        Rgba([blend(r), blend(g), blend(b), a])
    });

    Ok(DynamicImage::ImageRgba8(adjusted))
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, ImageBuffer, Rgba};

    use super::*;

    fn solid_rgba(r: u8, g: u8, b: u8, a: u8) -> DynamicImage {
        let mut buf = ImageBuffer::new(2, 2);
        for pixel in buf.pixels_mut() {
            *pixel = Rgba([r, g, b, a]);
        }
        DynamicImage::ImageRgba8(buf)
    }

    #[test]
    fn test_preprocess_sharp_preserves_dimensions() {
        let img = solid_rgba(128, 64, 32, 150);
        let result = preprocess(img, &MosaicMode::Sharp).unwrap();
        assert_eq!(result.dimensions(), (2, 2));
    }

    #[test]
    fn test_preprocess_smooth_preserves_dimensions() {
        let img = solid_rgba(128, 64, 32, 255);
        let result = preprocess(img, &MosaicMode::Smooth).unwrap();
        assert_eq!(result.dimensions(), (2, 2));
    }

    #[test]
    fn test_binarize_alpha_opaque() {
        let img = solid_rgba(255, 0, 0, 200);
        let result = binarize_alpha(img).into_rgba8();
        assert!(result.pixels().all(|p| p[3] == 255));
    }

    #[test]
    fn test_binarize_alpha_transparent() {
        let img = solid_rgba(255, 0, 0, 100);
        let result = binarize_alpha(img).into_rgba8();
        assert!(result.pixels().all(|p| p[3] == 0));
    }

    #[test]
    fn test_binarize_alpha_boundaries() {
        let img_127 = solid_rgba(255, 0, 0, 127);
        assert!(
            binarize_alpha(img_127)
                .into_rgba8()
                .pixels()
                .all(|p| p[3] == 0)
        );

        let img_128 = solid_rgba(255, 0, 0, 128);
        assert!(
            binarize_alpha(img_128)
                .into_rgba8()
                .pixels()
                .all(|p| p[3] == 255)
        );
    }

    #[test]
    fn test_apply_gaussian_blur_spreads_pixels() {
        let mut buf = ImageBuffer::new(5, 5);
        buf.put_pixel(2, 2, Rgba([255, 255, 255, 255]));
        let img = DynamicImage::ImageRgba8(buf);

        let blurred = apply_gaussian_blur(img, 1.0).unwrap().into_rgba8();

        assert!(blurred.get_pixel(2, 2)[0] < 255);
        assert!(blurred.get_pixel(2, 1)[0] > 0);
    }

    #[test]
    fn test_apply_saturation_grayscale() {
        // factor=0.0 should yield a grayscale image (all channels equal to luma)
        let img = solid_rgba(200, 100, 50, 255);
        let result = apply_saturation(img, 0.0).unwrap().into_rgba8();
        for p in result.pixels() {
            assert_eq!(p[0], p[1]);
            assert_eq!(p[1], p[2]);
        }
    }

    #[test]
    fn test_apply_saturation_no_change() {
        // factor=1.0 should preserve original colors
        let img = solid_rgba(200, 100, 50, 255);
        let result = apply_saturation(img, 1.0).unwrap().into_rgba8();
        for p in result.pixels() {
            assert_eq!(p[0], 200);
            assert_eq!(p[1], 100);
            assert_eq!(p[2], 50);
        }
    }

    #[test]
    fn test_apply_saturation_partial() {
        // luma = 255 * 0.2126 = 54.213
        // R = 54.213 + 0.5 * (255 - 54.213) = 154.6065 -> 155
        // G = 54.213 + 0.5 * (0 - 54.213) = 27.1065 -> 27
        // B = 54.213 + 0.5 * (0 - 54.213) = 27.1065 -> 27
        let img = solid_rgba(255, 0, 0, 255);
        let result = apply_saturation(img, 0.5).unwrap().into_rgba8();
        for p in result.pixels() {
            assert_eq!(p[0], 155);
            assert_eq!(p[1], 27);
            assert_eq!(p[2], 27);
        }
    }

    #[test]
    fn test_crop_to_square_landscape() {
        let buf = ImageBuffer::new(200, 100);
        let img = DynamicImage::ImageRgba8(buf);
        let cropped = crop_to_square(img, true);
        assert_eq!(cropped.dimensions(), (100, 100));
    }

    #[test]
    fn test_crop_to_square_portrait() {
        let buf = ImageBuffer::new(100, 200);
        let img = DynamicImage::ImageRgba8(buf);
        let cropped = crop_to_square(img, true);
        assert_eq!(cropped.dimensions(), (100, 100));
    }

    #[test]
    fn test_crop_to_square_disabled() {
        let buf = ImageBuffer::new(200, 100);
        let img = DynamicImage::ImageRgba8(buf);
        let result = crop_to_square(img, false);
        assert_eq!(result.dimensions(), (200, 100));
    }
}
