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
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = rgba.clone();

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let p = rgba.get_pixel(x, y);

            let Some([u, d, l, r, ul, ur, dl, dr]) = get_neighbors(&rgba, x, y) else {
                continue;
            };

            let target_color = [
                (l, u, r, d, ul), // Top-left L
                (r, u, l, d, ur), // Top-right L
                (l, d, r, u, dl), // Bottom-left L
                (r, d, l, u, dr), // Bottom-right L
            ]
            .into_iter()
            .find_map(|(adj1, adj2, opp1, opp2, diag)| {
                (adj1 == p && adj2 == p && opp1 != p && opp2 != p && diag != p).then(|| {
                    if opp1 == opp2 {
                        *opp1
                    } else {
                        let majority = find_majority_color(&rgba, x, y);
                        if majority != *p { majority } else { *opp1 }
                    }
                })
            });

            if let Some(c) = target_color {
                output.put_pixel(x, y, c);
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(output))
}

fn add_outlines(img: DynamicImage) -> Result<DynamicImage> {
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = rgba.clone();

    let outline_color = Rgba([0, 0, 0, 255]); // black

    for y in 0..height {
        for x in 0..width {
            if rgba.get_pixel(x, y)[3] > 0 {
                continue;
            }

            // If the current transparent pixel has an opaque neighbor, it's part of the outline.
            let is_outline_pixel = (y > 0 && rgba.get_pixel(x, y - 1)[3] > 0)
                || (y + 1 < height && rgba.get_pixel(x, y + 1)[3] > 0)
                || (x > 0 && rgba.get_pixel(x - 1, y)[3] > 0)
                || (x + 1 < width && rgba.get_pixel(x + 1, y)[3] > 0);

            if is_outline_pixel {
                output.put_pixel(x, y, outline_color);
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(output))
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

/// Retrieves references to the 8 neighboring pixels around the specified (x, y) coordinate.
/// Returns `None` if the coordinate is on the edge of the image and does not have all 8 neighbors.
#[inline]
fn get_neighbors(img: &image::RgbaImage, x: u32, y: u32) -> Option<[&Rgba<u8>; 8]> {
    let (width, height) = img.dimensions();
    if x == 0 || y == 0 || x >= width - 1 || y >= height - 1 {
        return None;
    }

    Some([
        img.get_pixel(x, y - 1),     // u
        img.get_pixel(x, y + 1),     // d
        img.get_pixel(x - 1, y),     // l
        img.get_pixel(x + 1, y),     // r
        img.get_pixel(x - 1, y - 1), // ul
        img.get_pixel(x + 1, y - 1), // ur
        img.get_pixel(x - 1, y + 1), // dl
        img.get_pixel(x + 1, y + 1), // dr
    ])
}
