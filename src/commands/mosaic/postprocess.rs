use image::{DynamicImage, Rgba};
use std::collections::HashMap;

use crate::Result;

use super::MosaicMode;

/// Post-processes the image after quantization.
pub fn postprocess(img: DynamicImage, mode: &MosaicMode) -> Result<DynamicImage> {
    match mode {
        MosaicMode::Character => {
            let img = remove_orphan_pixels(img)?;
            let img = apply_pixel_perfection(img)?;
            let img = add_outlines(img)?;
            let img = remove_anti_aliasing(img)?;
            Ok(img)
        }
        MosaicMode::Background => {
            let img = remove_anti_aliasing(img)?;
            Ok(img)
        }
    }
}

fn remove_orphan_pixels(img: DynamicImage) -> Result<DynamicImage> {
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = rgba.clone();

    for y in 0..height {
        for x in 0..width {
            if !is_orphan(&rgba, x, y) {
                continue;
            }

            let majority_color = find_majority_color(&rgba, x, y);
            output.put_pixel(x, y, majority_color);
        }
    }

    Ok(DynamicImage::ImageRgba8(output))
}

fn apply_pixel_perfection(img: DynamicImage) -> Result<DynamicImage> {
    Ok(img)
}

fn add_outlines(img: DynamicImage) -> Result<DynamicImage> {
    Ok(img)
}

fn remove_anti_aliasing(img: DynamicImage) -> Result<DynamicImage> {
    Ok(img)
}

fn is_orphan(img: &image::RgbaImage, x: u32, y: u32) -> bool {
    let (width, height) = img.dimensions();
    let center_pixel = img.get_pixel(x, y);

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                if img.get_pixel(nx as u32, ny as u32) == center_pixel {
                    return false;
                }
            }
        }
    }
    true // the pixel is orphan if it has no same colored neighbor
}

fn find_majority_color(img: &image::RgbaImage, x: u32, y: u32) -> Rgba<u8> {
    let (width, height) = img.dimensions();
    let mut counts: HashMap<Rgba<u8>, u32> = HashMap::with_capacity(8);

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let neighbor_pixel = img.get_pixel(nx as u32, ny as u32);
                *counts.entry(*neighbor_pixel).or_insert(0) += 1;
            }
        }
    }

    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(c, _)| c)
        .unwrap_or_else(|| *img.get_pixel(x, y))
}
