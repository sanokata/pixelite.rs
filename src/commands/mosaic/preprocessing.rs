use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
use libblur::{
    BlurImage, BlurImageMut, ConvolutionMode, EdgeMode2D, FastBlurChannels, GaussianBlurParams,
    ThreadingPolicy, gaussian_blur,
};

use super::MosaicMode;
use crate::Result;

/// Gaussian blur sigma applied during Background mode noise reduction.
const BACKGROUND_BLUR_SIGMA: f32 = 1.0;
/// Saturation scale factor for Background mode (values < 1.0 reduce saturation).
const BACKGROUND_SATURATION_FACTOR: f32 = 0.85;

/// Applies mode-appropriate preprocessing to the image before resizing and quantization.
///
/// - `Character`: Binarizes the alpha channel (0 or 255) to preserve hard edges.
/// - `Background`: Applies a gentle Gaussian blur to reduce noise and smooth gradients,
///   then lowers saturation slightly for more natural palette reduction.
pub fn preprocess(img: DynamicImage, mode: &MosaicMode) -> Result<DynamicImage> {
    match mode {
        MosaicMode::Character => preprocess_character(img),
        MosaicMode::Background => preprocess_background(img),
    }
}

/// Character mode: keep edges intact; only binarize alpha so quantization
/// does not create semi-transparent fringe pixels.
fn preprocess_character(img: DynamicImage) -> Result<DynamicImage> {
    // infalliable but returning Result for API uniformity with preprocess_background
    Ok(binarize_alpha(img))
}

/// Background mode: smooth noise with a Gaussian blur, then reduce saturation
/// so that the KMeans palette covers gradient areas more evenly.
fn preprocess_background(img: DynamicImage) -> Result<DynamicImage> {
    let img = apply_gaussian_blur(img, BACKGROUND_BLUR_SIGMA)?;
    let img = apply_saturation(img, BACKGROUND_SATURATION_FACTOR)?;
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
    // TODO
}
